use super::*;

use std::path::Path;
use std::time::Duration;

use versionlens_model::document_text_hash;

#[test]
fn two_new_unsaved_members_share_one_discovered_snapshot() {
    let workspace = TestWorkspace::new("session-workspace");
    let root = workspace.document(
        "package.json",
        r#"{"workspaces":["packages/*"],"packageManager":"pnpm@10.0.0"}"#,
        1,
    );
    let lerna = workspace.document("lerna.json", r#"{"version":"independent"}"#, 4);
    let a = workspace.document(
        "packages/a/package.json",
        r#"{"name":"a","version":"1.0.0"}"#,
        2,
    );
    let b = workspace.document(
        "packages/b/package.json",
        r#"{"name":"b","version":"2.0.0"}"#,
        3,
    );
    let session = session();
    assert!(session.set_workspace_documents(vec![root, a.clone(), b, lerna]));

    let first = session.workspace_graph(&a);
    session.workspace_graph(&a);
    let resolved = first.resolve(&dependency("b", "workspace:*")).unwrap();
    assert_eq!(resolved.version, "2.0.0");
    assert_eq!(
        resolved.policy,
        crate::workspace::WorkspacePolicy::Independent
    );
    assert!(first.resolve(&dependency("b", "^2.0.0")).is_none());
    assert_cached_graphs(&session, 1);
}

#[test]
fn clean_disk_documents_reuse_the_generation_graph() {
    let workspace = TestWorkspace::new("session-workspace");
    let text = r#"{"name":"a","version":"1.0.0"}"#;
    let b_text = r#"{"name":"b","version":"2.0.0"}"#;
    workspace.write_all(&[
        ("package.json", r#"{"workspaces":["packages/*"]}"#),
        ("packages/a/package.json", text),
        ("packages/b/package.json", b_text),
    ]);
    let a = workspace.document("packages/a/package.json", text, 7);
    let b = workspace.document("packages/b/package.json", b_text, 8);
    let session = session();

    for input in [&a, &b] {
        session.workspace_graph(input);
    }
    assert_cached_graphs(&session, 1);
}

#[test]
fn non_graph_document_reuses_cached_workspace_graph() {
    let workspace = TestWorkspace::new("session-workspace-non-graph");
    let first_member = r#"{"name":"a","version":"1.0.0"}"#;
    workspace.write_all(&[
        ("package.json", r#"{"workspaces":["packages/*"]}"#),
        ("packages/a/package.json", first_member),
    ]);
    let member = workspace.document("packages/a/package.json", first_member, 1);
    let session = session();
    let first = session.workspace_graph(&member);
    assert_workspace_version(&first, "a", "workspace:*", "1.0.0");

    workspace.write(
        "packages/a/package.json",
        r#"{"name":"a","version":"2.0.0"}"#,
    );
    let unrelated = workspace.document("requirements.txt", "demo==1.0.0", 2);
    let reused = session.workspace_graph(&unrelated);

    assert_workspace_version(&reused, "a", "workspace:*", "1.0.0");
}

#[test]
fn changed_disk_member_invalidates_persistent_local_suggestion_freshness() {
    let (workspace, consumer) = local_member_workspace("persistent-local-member");
    let cache =
        versionlens_test_support::temporary_directory("versionlens-workspace-cache").unwrap();
    let first = crate::support::tests::test_session(false)
        .with_persistent_cache(&cache)
        .unwrap();
    let local_dependency = local_dependency(&first, &consumer);
    assert_eq!(local_dependency.name, "local-package");
    assert_eq!(local_dependency.requirement, "^1.0.0");
    let local = first
        .workspace_graph(&consumer)
        .resolve(&local_dependency)
        .unwrap();
    assert_eq!(local.version, "1.0.0");
    let context = npm_context(&first, &consumer);
    let scope = first.document_cache_scope(&context, &consumer);
    let operation = first.operation_context();
    first.cache_resolved_suggestions(
        &[versionlens_suggestions::resolve_dependency(
            local_dependency,
            Some(local.version),
        )],
        Some(ManifestKind::NpmPackageJson),
        &operation,
        &scope,
    );
    drop(first);

    workspace.write(
        "packages/local/package.json",
        r#"{"name":"local-package","version":"2.0.0"}"#,
    );
    let restarted = crate::support::tests::test_session(false)
        .with_persistent_cache(&cache)
        .unwrap();
    assert!(!restarted.document_is_fresh(&consumer));

    std::fs::remove_dir_all(cache).unwrap();
}

#[test]
fn replacement_and_disk_generations_reject_stale_publication() {
    let workspace = TestWorkspace::new("session-workspace");
    let root = workspace.document("package.json", r#"{"workspaces":[]}"#, 1);
    let session = session();
    let clone = session.clone();
    let before_replace = session.operation_context();
    assert!(clone.set_workspace_documents(vec![root.clone()]));
    assert!(!before_replace.can_publish());

    let unchanged = session.operation_context();
    assert!(!session.set_workspace_documents(vec![root]));
    assert!(unchanged.can_publish());

    let before_disk_change = session.operation_context();
    clone.invalidate_workspace();
    assert!(!before_disk_change.can_publish());
}

#[test]
fn invalidation_rebuilds_graphs_and_clears_only_suggestion_freshness() {
    let (_workspace, session, a) = registered_member(r#"{"name":"a","version":"1.0.0"}"#, 2);
    session.workspace_graph(&a);
    assert_cached_graphs(&session, 1);
    let cached_dependency = dependency("cached", "1.0.0");
    let key = crate::cache::suggestion_cache_key(&cached_dependency, "workspace-test");
    let latest_key = crate::cache::latest_cache_key(&cached_dependency);
    session.suggestion_cache().insert_with_ttl(
        key.clone(),
        versionlens_suggestions::fixed(cached_dependency, "1.0.0".to_owned()),
        Duration::from_secs(60),
    );
    session.cache().insert_with_ttl(
        latest_key.clone(),
        crate::session::cache::CachedLatest {
            latest: "1.0.0".to_owned(),
            builds: Vec::new(),
            choices: Vec::new(),
            fixed_requirement_matched: Some(false),
        },
        Duration::from_secs(60),
    );

    session.invalidate_workspace();
    assert_cached_graphs(&session, 0);
    session.workspace_graph(&a);
    assert_cached_graphs(&session, 1);
    assert!(session.suggestion_cache().get(&key).is_none());
    assert!(session.cache().get(&latest_key).is_some());
}

#[test]
fn explicit_cache_clear_discards_graphs_but_retains_overlays() {
    let (_workspace, session, a) = registered_member(r#"{"name":"a","version":"1.0.0"}"#, 2);
    session.workspace_graph(&a);
    assert_cached_graphs(&session, 1);

    session.clear_cache();
    assert_cached_graphs(&session, 0);
    let second = session.workspace_graph(&a);
    assert_cached_graphs(&session, 1);
    assert!(second.resolve(&dependency("a", "workspace:*")).is_some());
}

#[test]
fn differing_current_input_is_a_coherent_temporary_overlay() {
    let (workspace, session, saved) = registered_member(r#"{"name":"a","version":"1.0.0"}"#, 2);
    let changed = workspace.document(
        "packages/a/package.json",
        r#"{"name":"a","version":"3.0.0"}"#,
        3,
    );
    session.workspace_graph(&saved);
    let temporary = session.workspace_graph(&changed);

    assert_workspace_version(&temporary, "a", "workspace:*", "3.0.0");
    assert_cached_graphs(&session, 1);
    assert_workspace_version(
        &session.workspace_graph(&saved),
        "a",
        "workspace:*",
        "1.0.0",
    );
}

#[test]
fn coordinated_plan_uses_versions_and_hashes_from_all_overlays() {
    let workspace = TestWorkspace::new("session-workspace");
    let root = workspace.document("package.json", r#"{"workspaces":["packages/*"]}"#, 1);
    let a_text = r#"{"name":"a","version":"1.0.0"}"#;
    let b_text = r#"{"name":"b","version":"1.0.0","dependencies":{"a":"^1.0.0"}}"#;
    let a = workspace.document("packages/a/package.json", a_text, 11);
    let b = workspace.document("packages/b/package.json", b_text, 12);
    let session = session();
    session.set_workspace_documents(vec![root, a.clone(), b.clone()]);

    let plan = session
        .workspace_graph(&a)
        .coordinated_plan(&a, &[], "a", "2.0.0")
        .unwrap()
        .unwrap();
    assert_snapshot(&plan, &a, a_text, 11);
    assert_snapshot(&plan, &b, b_text, 12);
}

fn assert_snapshot(
    plan: &versionlens_model::WorkspaceEditPlan,
    input: &DocumentInput,
    text: &str,
    version: u64,
) {
    let path = document_path(input).unwrap();
    let document = plan
        .documents
        .iter()
        .find(|document| Path::new(&document.document.uri) == path)
        .unwrap();
    assert_eq!(document.document.version, Some(version));
    assert_eq!(document.document.text_hash, document_text_hash(text));
}
