use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::PoisonError;
use std::sync::{Arc, atomic::Ordering};

use versionlens_model::{DocumentInput, ManifestKind, document_text_hash};

use crate::registry::{
    RegistryContext, RegistryFileSnapshot, registry_context_from_document_kind_with_files,
};
use crate::workspace::{
    WorkspaceDocuments, WorkspaceGraph, document_path, workspace_path, workspace_root,
};
use crate::{VersionLensSession, WorkspaceDiscovery, WorkspaceDiscoveryOptions};

const MAX_REGISTRY_CONTEXTS: usize = 256;

#[derive(Debug, Default)]
pub(crate) struct WorkspaceState {
    generation: u64,
    documents: Arc<WorkspaceDocuments>,
    exclusions: Arc<[String]>,
    registry_documents: Arc<WorkspaceDocuments>,
    graphs: BTreeMap<PathBuf, WorkspaceGraph>,
    registry_files: RegistryFileSnapshot,
    registry_contexts: BTreeMap<RegistryContextKey, RegistryContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RegistryContextKey {
    uri: String,
    workspace_root: Option<String>,
    manifest_kind: String,
    text_hash: String,
}

impl RegistryContextKey {
    fn new(input: &DocumentInput, kind: ManifestKind) -> Self {
        Self {
            uri: input.uri.clone(),
            workspace_root: input.workspace_root.clone(),
            manifest_kind: format!("{kind:?}"),
            text_hash: document_text_hash(&input.text),
        }
    }
}

impl VersionLensSession {
    /// Replaces the complete set of unsaved workspace documents.
    ///
    /// Invalid URIs and documents outside their declared workspace root do not
    /// participate in workspace discovery.
    pub fn set_workspace_documents(&self, documents: Vec<DocumentInput>) -> bool {
        let mut workspace_documents = WorkspaceDocuments::new();
        let mut registry_documents = WorkspaceDocuments::new();
        for document in documents {
            let workspace_document_path = document_path(&document);
            let registry_document_path = workspace_document_path.clone().or_else(|| {
                document
                    .workspace_root
                    .is_none()
                    .then(|| workspace_path(&document.uri))
                    .flatten()
            });
            if let Some(path) = workspace_document_path {
                workspace_documents.insert(path, (document.text.clone(), document.version));
            }
            if let Some(path) = registry_document_path {
                registry_documents.insert(path, (document.text, document.version));
            }
        }
        let mut state = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if state.documents.as_ref() == &workspace_documents
            && state.registry_documents.as_ref() == &registry_documents
        {
            return false;
        }
        state.documents = Arc::new(workspace_documents);
        state.registry_documents = Arc::new(registry_documents);
        // Graphs and registry contexts are rebuilt from the new text. Resolved
        // suggestions stay: their cache keys carry the registry context and the
        // workspace fingerprint, so an edit that changes either misses them.
        rebuild_graphs(&mut state);
        drop(state);
        true
    }

    /// Replaces the globs, relative to a workspace root, of the paths that
    /// never become workspace members. Ignore files apply as well.
    pub fn set_workspace_exclusions(&self, exclusions: Vec<String>) {
        let mut state = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if *state.exclusions != *exclusions {
            state.exclusions = exclusions.into();
            rebuild_graphs(&mut state);
        }
        drop(state);
    }

    /// The skip rules of `root`: its ignore files and the workspace
    /// exclusions. It is only asked about paths, never iterated.
    pub(crate) fn workspace_filter(&self, root: &Path) -> WorkspaceDiscovery {
        let mut options = WorkspaceDiscoveryOptions::new(vec![root.to_path_buf()]);
        options.exclusions = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .exclusions
            .to_vec();
        self.discover_workspace_documents(options)
    }

    /// Signals that workspace files or workspace configuration changed on disk.
    pub fn invalidate_workspace(&self) {
        let mut state = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        invalidate_state(self, &mut state);
        self.suggestion_cache().clear();
        drop(state);
    }

    pub(crate) fn clear_workspace_graphs(&self) {
        let mut state = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        state.generation = state.generation.wrapping_add(1);
        state.graphs.clear();
        state.registry_files = RegistryFileSnapshot::new(Arc::clone(&state.registry_documents));
        state.registry_contexts.clear();
    }

    /// Changes each time the unsaved documents or the files on disk change.
    pub(crate) fn workspace_revision(&self) -> u64 {
        self.storage_state
            .workspace
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .generation
    }

    pub(crate) fn workspace_documents(&self) -> Arc<WorkspaceDocuments> {
        Arc::clone(
            &self
                .storage_state
                .workspace
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .documents,
        )
    }

    pub(crate) fn workspace_graph(&self, input: &DocumentInput) -> WorkspaceGraph {
        let Some(root) = workspace_root(input) else {
            return WorkspaceGraph::default();
        };
        let (generation, documents, cached) = {
            let state = self
                .storage_state
                .workspace
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            (
                state.generation,
                Arc::clone(&state.documents),
                state.graphs.get(&root).cloned(),
            )
        };

        let mut discovery = None;
        let mut skips = |path: &Path, directory: bool| {
            let discovery = discovery.get_or_insert_with(|| self.workspace_filter(&root));
            if directory {
                discovery.skips_directory(path)
            } else {
                discovery.skips_file(path)
            }
        };
        let graph = if let Some(graph) = cached {
            graph
        } else {
            let graph = WorkspaceGraph::for_workspace(&root, &documents, &mut skips);
            let mut state = self
                .storage_state
                .workspace
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let graph = if state.generation == generation {
                state.graphs.entry(root.clone()).or_insert(graph).clone()
            } else {
                graph
            };
            drop(state);
            graph
        };
        if graph.captures(input) || !workspace_input_affects_graph(input) {
            return graph.bind_source(input);
        }

        let mut documents = documents.as_ref().clone();
        if let Some(current) = document_path(input) {
            documents.insert(current, (input.text.clone(), input.version));
        }
        WorkspaceGraph::for_workspace(&root, &documents, &mut skips).bind_source(input)
    }

    pub(crate) fn registry_context(
        &self,
        input: &DocumentInput,
        kind: ManifestKind,
    ) -> RegistryContext {
        let key = RegistryContextKey::new(input, kind);
        let (generation, files) = {
            let state = self
                .storage_state
                .workspace
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if let Some(context) = state.registry_contexts.get(&key) {
                return context.clone();
            }
            (state.generation, state.registry_files.clone())
        };
        let reader = files.reader(input);
        let context = registry_context_from_document_kind_with_files(input, kind, &reader);
        let mut state = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if state.generation != generation {
            return context;
        }
        if state.registry_contexts.len() >= MAX_REGISTRY_CONTEXTS
            && !state.registry_contexts.contains_key(&key)
        {
            state.registry_contexts.pop_first();
        }
        state
            .registry_contexts
            .entry(key)
            .or_insert(context)
            .clone()
    }
}

fn workspace_input_affects_graph(input: &DocumentInput) -> bool {
    document_path(input).is_some_and(|path| {
        matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("package.json" | "Cargo.toml" | "pnpm-workspace.yaml" | "lerna.json")
        )
    })
}

fn rebuild_graphs(state: &mut WorkspaceState) {
    state.generation = state.generation.wrapping_add(1);
    state.graphs.clear();
    state.registry_files = RegistryFileSnapshot::new(Arc::clone(&state.registry_documents));
    state.registry_contexts.clear();
}

fn invalidate_state(session: &VersionLensSession, state: &mut WorkspaceState) {
    rebuild_graphs(state);
    session
        .storage_state
        .cache_epoch
        .fetch_add(1, Ordering::AcqRel);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProviderSettings;
    use std::sync::PoisonError;
    use versionlens_model::{Dependency, Ecosystem, ManifestKind};

    use crate::workspace::tests::support::{TestWorkspace, dependency};

    const LOCAL_CONSUMER_TEXT: &str = r#"{
  "name": "consumer",
  "version": "1.0.0",
  "dependencies": {
    "local-package": "^1.0.0"
  }
}"#;

    fn session() -> VersionLensSession {
        VersionLensSession::new(crate::test_support::session_config(
            ProviderSettings::default(),
            false,
        ))
    }

    fn cached_graphs(session: &VersionLensSession) -> usize {
        session
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .graphs
            .len()
    }

    fn assert_cached_graphs(session: &VersionLensSession, expected: usize) {
        assert_eq!(cached_graphs(session), expected);
    }

    fn registry_url(context: &RegistryContext, name: &str) -> String {
        context
            .registry_endpoints(&dependency(name, "1.0.0"))
            .into_iter()
            .next()
            .expect("registry endpoint")
            .url
    }

    fn npm_input(workspace: &TestWorkspace, relative: &str) -> DocumentInput {
        workspace.document(
            relative,
            r#"{"name":"app","dependencies":{"demo":"1.0.0"}}"#,
            1,
        )
    }

    fn npm_document(uri: String, workspace_root: String) -> DocumentInput {
        DocumentInput::new(
            uri,
            "json",
            r#"{"dependencies":{"demo":"1.0.0"}}"#,
            Some(workspace_root),
        )
    }

    fn rootless(mut input: DocumentInput) -> DocumentInput {
        let path = workspace_path(&input.uri).expect("rootless document path");
        input.uri = crate::workspace_file_uri(&path).expect("rootless document URI");
        input.workspace_root = None;
        input
    }

    fn npm_context(session: &VersionLensSession, input: &DocumentInput) -> RegistryContext {
        session.registry_context(input, ManifestKind::NpmPackageJson)
    }

    fn npm_context_with_documents(
        input: &DocumentInput,
        documents: Vec<DocumentInput>,
    ) -> RegistryContext {
        let session = session();
        assert!(session.set_workspace_documents(documents));
        npm_context(&session, input)
    }

    fn registered_member(
        text: &str,
        version: u64,
    ) -> (TestWorkspace, VersionLensSession, DocumentInput) {
        let workspace = TestWorkspace::new("session-workspace");
        let root = workspace.document("package.json", r#"{"workspaces":["packages/*"]}"#, 1);
        let member = workspace.document("packages/a/package.json", text, version);
        let session = session();
        session.set_workspace_documents(vec![root, member.clone()]);
        (workspace, session, member)
    }

    fn local_member_workspace(name: &str) -> (TestWorkspace, DocumentInput) {
        let workspace = TestWorkspace::new(name);
        workspace.write_all(&[
            ("package.json", r#"{"workspaces":["packages/*"]}"#),
            (
                "packages/local/package.json",
                r#"{"name":"local-package","version":"1.0.0"}"#,
            ),
            ("packages/consumer/package.json", LOCAL_CONSUMER_TEXT),
        ]);
        let mut consumer =
            workspace.document("packages/consumer/package.json", LOCAL_CONSUMER_TEXT, 1);
        consumer.uri =
            crate::workspace_file_uri(&workspace.root.join("packages/consumer/package.json"))
                .unwrap();
        (workspace, consumer)
    }

    fn local_dependency(session: &VersionLensSession, consumer: &DocumentInput) -> Dependency {
        session
            .dependencies(consumer)
            .into_iter()
            .find(|dependency| dependency.name == "local-package")
            .unwrap()
    }

    fn assert_workspace_version(
        graph: &WorkspaceGraph,
        name: &str,
        requirement: &str,
        expected: &str,
    ) {
        assert_eq!(
            graph
                .resolve(&dependency(name, requirement))
                .unwrap()
                .version,
            expected
        );
    }

    fn registry_configuration(
        workspace: &TestWorkspace,
        host: &str,
        version: u64,
        rootless_config: bool,
    ) -> DocumentInput {
        let configuration = workspace.document(
            ".npmrc",
            &format!("registry=https://{host}/\n//{host}/:_authToken={version}"),
            version,
        );
        if rootless_config {
            rootless(configuration)
        } else {
            configuration
        }
    }

    fn assert_registry_overlay_replacement(
        workspace: &TestWorkspace,
        input: &DocumentInput,
        hosts: [&str; 2],
        rootless_config: bool,
    ) {
        let session = session();
        let first = registry_configuration(workspace, hosts[0], 1, rootless_config);
        session.set_workspace_documents(vec![first]);
        let first_context = npm_context(&session, input);
        let first_scope = session.document_cache_scope(&first_context, input);

        let second = registry_configuration(workspace, hosts[1], 2, rootless_config);
        assert!(session.set_workspace_documents(vec![second]));
        let second_context = npm_context(&session, input);
        let second_scope = session.document_cache_scope(&second_context, input);

        assert!(
            registry_url(&first_context, "demo").starts_with(&format!("https://{}/", hosts[0]))
        );
        assert!(
            registry_url(&second_context, "demo").starts_with(&format!("https://{}/", hosts[1]))
        );
        assert_ne!(first_scope, second_scope);
        assert_ne!(
            first_context
                .auth_headers_for_url(Ecosystem::Npm, &format!("https://{}/demo", hosts[0]),),
            second_context
                .auth_headers_for_url(Ecosystem::Npm, &format!("https://{}/demo", hosts[1]),)
        );
    }

    fn assert_registry_size_failure(session: &VersionLensSession, input: &DocumentInput) {
        assert!(
            npm_context(session, input)
                .failure_message()
                .is_some_and(|message| message.contains("the limit is"))
        );
    }

    mod graphs {
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
                versionlens_test_support::temporary_directory("versionlens-workspace-cache")
                    .unwrap();
            let first = crate::test_support::test_session(false)
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
                &[versionlens_update::resolve_dependency(
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
            let restarted = crate::test_support::test_session(false)
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
            // An unsaved edit keeps resolved state: cache keys already carry the
            // workspace fingerprint, so results of earlier work stay valid.
            let before_replace = session.operation_context();
            assert!(clone.set_workspace_documents(vec![root.clone()]));
            assert!(before_replace.can_publish());

            let unchanged = session.operation_context();
            assert!(!session.set_workspace_documents(vec![root]));
            assert!(unchanged.can_publish());

            let before_disk_change = session.operation_context();
            clone.invalidate_workspace();
            assert!(!before_disk_change.can_publish());
        }

        #[test]
        fn invalidation_rebuilds_graphs_and_clears_only_suggestion_freshness() {
            let (_workspace, session, a) =
                registered_member(r#"{"name":"a","version":"1.0.0"}"#, 2);
            session.workspace_graph(&a);
            assert_cached_graphs(&session, 1);
            let cached_dependency = dependency("cached", "1.0.0");
            let key = crate::cache::suggestion_cache_key(&cached_dependency, "workspace-test");
            let latest_key = crate::cache::latest_cache_key(&cached_dependency);
            session.suggestion_cache().insert_with_ttl(
                key.clone(),
                versionlens_update::fixed(cached_dependency, "1.0.0".to_owned()),
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
        fn overlay_edits_rebuild_graphs_but_keep_resolved_suggestions() {
            let (workspace, session, member) =
                registered_member(r#"{"name":"a","version":"1.0.0"}"#, 2);
            session.workspace_graph(&member);
            assert_cached_graphs(&session, 1);
            let cached_dependency = dependency("cached", "1.0.0");
            let key = crate::cache::suggestion_cache_key(&cached_dependency, "workspace-test");
            session.suggestion_cache().insert_with_ttl(
                key.clone(),
                versionlens_update::fixed(cached_dependency, "1.0.0".to_owned()),
                Duration::from_secs(60),
            );
            let root = workspace.document("package.json", r#"{"workspaces":["packages/*"]}"#, 1);
            let edited = workspace.document(
                "packages/a/package.json",
                "{\n  // edited\n  \"name\":\"a\",\"version\":\"1.0.0\"}",
                3,
            );

            assert!(session.set_workspace_documents(vec![root, edited]));
            assert_cached_graphs(&session, 0);
            assert!(session.suggestion_cache().get(&key).is_some());
        }

        #[test]
        fn explicit_cache_clear_discards_graphs_but_retains_overlays() {
            let (_workspace, session, a) =
                registered_member(r#"{"name":"a","version":"1.0.0"}"#, 2);
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
            let (workspace, session, saved) =
                registered_member(r#"{"name":"a","version":"1.0.0"}"#, 2);
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

        #[test]
        fn ignored_and_excluded_copies_are_not_members() {
            let workspace = TestWorkspace::new("session-workspace");
            let member = r#"{"name":"a","version":"1.0.0"}"#;
            workspace.write_all(&[
                (".gitignore", ".claude/worktrees/\n"),
                ("package.json", r#"{"workspaces":["**"]}"#),
                ("packages/a/package.json", member),
                (".claude/worktrees/copy/packages/a/package.json", member),
                ("generated/a/package.json", member),
            ]);
            let input = workspace.document("package.json", r#"{"workspaces":["**"]}"#, 1);
            let session = session();

            session.set_workspace_exclusions(vec!["**/generated".to_owned()]);
            let graph = session.workspace_graph(&input);
            assert_workspace_version(&graph, "a", "^1.0.0", "1.0.0");

            session.set_workspace_exclusions(Vec::new());
            assert_eq!(cached_graphs(&session), 0);
            assert!(
                session
                    .workspace_graph(&input)
                    .resolve(&dependency("a", "^1.0.0"))
                    .is_none()
            );
        }
    }

    mod registry {
        use super::*;
        use std::sync::PoisonError;

        #[test]
        fn registry_context_reads_new_npm_yarn_nuget_and_gradle_overlays() {
            let workspace = TestWorkspace::new("registry-overlays");
            let npm = npm_input(&workspace, "npm/package.json");
            let yarn = npm_input(&workspace, "yarn/package.json");
            let dotnet = workspace.document("dotnet/app.csproj", "<Project />", 1);
            let gradle = workspace.document("gradle/build.gradle", "", 1);
            let configs = vec![
        workspace.document("npm/.npmrc", "registry=https://npm.overlay.test/", 2),
        workspace.document(
            "yarn/.yarnrc.yml",
            "npmRegistryServer: https://yarn.overlay.test/",
            3,
        ),
        workspace.document(
            "dotnet/NuGet.config",
            r#"<configuration><packageSources><clear /><add key="private" value="https://nuget.overlay.test/v3/index.json" /></packageSources></configuration>"#,
            4,
        ),
        workspace.document(
            "gradle/settings.gradle",
            r#"dependencyResolutionManagement {
    repositories {
        maven {
            url = uri("https://gradle.overlay.test/releases")
        }
    }
}"#,
            5,
        ),
    ];
            let session = session();
            assert!(session.set_workspace_documents(configs));

            let npm_registry = npm_context(&session, &npm);
            assert!(registry_url(&npm_registry, "demo").starts_with("https://npm.overlay.test/"));
            let yarn_context = npm_context(&session, &yarn);
            assert!(registry_url(&yarn_context, "demo").starts_with("https://yarn.overlay.test/"));

            let dotnet_context = session.registry_context(&dotnet, ManifestKind::DotnetXml);
            let mut dotnet_dependency = dependency("Demo", "1.0.0");
            dotnet_dependency.ecosystem = Ecosystem::Dotnet;
            assert_eq!(
                dotnet_context.dotnet_registry_urls(&dotnet_dependency),
                ["https://nuget.overlay.test/v3/index.json"]
            );

            let gradle_context = session.registry_context(&gradle, ManifestKind::GradleBuild);
            assert!(
                gradle_context
                    .urls
                    .iter()
                    .any(|config| config.url == "https://gradle.overlay.test/releases")
            );
        }

        #[test]
        fn rootless_manifests_share_unsaved_registry_overlays_without_workspace_graphs() {
            let first = TestWorkspace::new("rootless-registry-first");
            let second = TestWorkspace::new("rootless-registry-second");
            let first_input = rootless(npm_input(&first, "package.json"));
            let second_input = rootless(npm_input(&second, "package.json"));
            let documents = vec![
                rootless(first.document(".npmrc", "registry=https://first-rootless.test/", 2)),
                rootless(second.document(".npmrc", "registry=https://second-rootless.test/", 3)),
            ];
            let session = session();

            assert!(session.set_workspace_documents(documents));
            assert_eq!(session.workspace_documents().len(), 0);
            assert!(
                registry_url(&npm_context(&session, &first_input), "demo")
                    .starts_with("https://first-rootless.test/")
            );
            assert!(
                registry_url(&npm_context(&session, &second_input), "demo")
                    .starts_with("https://second-rootless.test/")
            );
        }

        #[test]
        fn rootless_registry_overlay_changes_replace_context_and_auth_scope() {
            let workspace = TestWorkspace::new("rootless-registry-generation");
            let input = rootless(npm_input(&workspace, "package.json"));
            assert_registry_overlay_replacement(
                &workspace,
                &input,
                ["first-rootless.test", "second-rootless.test"],
                true,
            );
        }

        #[test]
        fn rootless_cargo_manifest_reads_unsaved_cargo_registry_config() {
            let workspace = TestWorkspace::new("rootless-cargo-registry");
            let input =
                rootless(workspace.document("Cargo.toml", "[dependencies]\nserde = \"1\"", 1));
            let config = rootless(workspace.document(
        ".cargo/config.toml",
        "[source.crates-io]\nreplace-with = 'mirror'\n[source.mirror]\nregistry = 'sparse+https://cargo-rootless.test/api/'",
        2,
    ));
            let session = session();
            assert!(session.set_workspace_documents(vec![config]));
            let context = session.registry_context(&input, ManifestKind::CargoToml);
            let mut cargo_dependency = dependency("serde", "1");
            cargo_dependency.ecosystem = Ecosystem::Cargo;

            assert_eq!(
                context.registry_endpoints(&cargo_dependency)[0].url,
                "https://cargo-rootless.test/api/serde/versions"
            );
        }

        #[test]
        fn rootless_registry_overlays_keep_the_registry_file_size_limit() {
            let workspace = TestWorkspace::new("rootless-registry-bound");
            let input = rootless(npm_input(&workspace, "package.json"));
            let oversized = rootless(workspace.document(".npmrc", &"x".repeat(1024 * 1024 + 1), 1));
            let session = session();
            assert!(session.set_workspace_documents(vec![oversized]));

            assert_registry_size_failure(&session, &input);

            let recovered = rootless(workspace.document(
                ".npmrc",
                "registry=https://rootless-recovered.test/",
                2,
            ));
            assert!(session.set_workspace_documents(vec![recovered]));
            let context = npm_context(&session, &input);
            assert!(context.failure_message().is_none());
            assert!(registry_url(&context, "demo").starts_with("https://rootless-recovered.test/"));
        }

        #[test]
        fn escaped_document_uri_finds_an_unsaved_sibling_config() {
            let workspace = TestWorkspace::new("registry-uri");
            let directory = "project space #1";
            let config = workspace.document(
                &format!("{directory}/.npmrc"),
                "registry=https://escaped.overlay.test/",
                2,
            );
            let path = workspace.root.join(directory).join("package.json");
            let encoded_uri = format!(
                "file://{}",
                path.to_string_lossy()
                    .replace(' ', "%20")
                    .replace('#', "%23")
            );
            let input = npm_document(encoded_uri, workspace.root.to_string_lossy().into_owned());
            let session = session();
            session.set_workspace_documents(vec![config]);

            let context = npm_context(&session, &input);
            assert!(registry_url(&context, "demo").starts_with("https://escaped.overlay.test/"));
        }

        #[cfg(unix)]
        #[test]
        fn symlink_workspace_root_normalizes_a_new_unsaved_config() {
            use std::os::unix::fs::symlink;

            let workspace = TestWorkspace::new("registry-symlink-target");
            let alias_holder = TestWorkspace::new("registry-symlink-alias");
            let alias = alias_holder.root.join("workspace");
            symlink(&workspace.root, &alias).unwrap();
            let root = alias.to_string_lossy().into_owned();
            let config = DocumentInput::new(
                alias.join("new/.npmrc").to_string_lossy(),
                "properties",
                "registry=https://symlink.overlay.test/",
                Some(root.clone()),
            )
            .with_version(2);
            let input = npm_document(
                alias
                    .join("new/package.json")
                    .to_string_lossy()
                    .into_owned(),
                root,
            );
            let context = npm_context_with_documents(&input, vec![config]);
            assert!(registry_url(&context, "demo").starts_with("https://symlink.overlay.test/"));
        }

        #[test]
        fn registry_overlay_changes_replace_context_and_cache_scope() {
            let workspace = TestWorkspace::new("registry-generation");
            let input = npm_input(&workspace, "package.json");
            assert_registry_overlay_replacement(
                &workspace,
                &input,
                ["first.test", "second.test"],
                false,
            );
        }

        #[test]
        fn disk_registry_context_is_stable_until_workspace_invalidation() {
            let workspace = TestWorkspace::new("registry-disk-cache");
            let input = npm_input(&workspace, "package.json");
            let session = session();

            let missing = npm_context(&session, &input);
            assert_eq!(
                missing
                    .registry_endpoints(&dependency("demo", "1.0.0"))
                    .len(),
                0
            );
            workspace.write(".npmrc", "registry=https://before.test/");
            let cached_miss = npm_context(&session, &input);
            assert_eq!(
                cached_miss
                    .registry_endpoints(&dependency("demo", "1.0.0"))
                    .len(),
                0
            );

            session.invalidate_workspace();
            let before = npm_context(&session, &input);
            workspace.write(".npmrc", "registry=https://after.test/");
            let cached_content = npm_context(&session, &input);
            assert_eq!(
                registry_url(&before, "demo"),
                registry_url(&cached_content, "demo")
            );

            session.invalidate_workspace();
            let after = npm_context(&session, &input);
            assert!(registry_url(&after, "demo").starts_with("https://after.test/"));
        }

        #[test]
        fn parsed_registry_context_cache_evicts_old_input_snapshots() {
            let workspace = TestWorkspace::new("registry-context-bound");
            let session = session();
            for version in 0..=MAX_REGISTRY_CONTEXTS {
                let input = workspace.document(
                    "package.json",
                    &format!(r#"{{"name":"app","version":"{version}"}}"#),
                    version as u64,
                );
                npm_context(&session, &input);
            }

            let state = session
                .storage_state
                .workspace
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            assert_eq!(state.registry_contexts.len(), MAX_REGISTRY_CONTEXTS);
        }

        #[test]
        fn sparse_oversized_registry_file_fails_until_workspace_invalidation() {
            let workspace = TestWorkspace::new("registry-sparse-bound");
            let input = npm_input(&workspace, "package.json");
            let config_path = workspace.root.join(".npmrc");
            let file = std::fs::File::create(&config_path).unwrap();
            file.set_len(1024 * 1024 + 1).unwrap();
            let session = session();

            assert_registry_size_failure(&session, &input);
            std::fs::write(&config_path, "registry=https://recovered.test/").unwrap();
            assert!(npm_context(&session, &input).failure_message().is_some());

            session.invalidate_workspace();
            let recovered = npm_context(&session, &input);
            assert!(recovered.failure_message().is_none());
            assert!(registry_url(&recovered, "demo").starts_with("https://recovered.test/"));
        }

        #[tokio::test]
        async fn registry_failure_does_not_replace_an_unrelated_cached_context() {
            let workspace = TestWorkspace::new("registry-failure-cache");
            workspace.write("healthy/.npmrc", "registry=https://cached.test/");
            let mut input = npm_input(&workspace, "healthy/package.json");
            input.uri =
                crate::workspace_file_uri(&workspace.root.join("healthy/package.json")).unwrap();
            let session = crate::test_support::test_session(false);
            let successful = session
                .resolve_document_with_responses(
                    input.clone(),
                    &[crate::RegistryResponseInput::new(
                        "demo".to_owned(),
                        Ecosystem::Npm,
                        r#"{"versions":{"1.0.0":{},"2.0.0":{}}}"#.to_owned(),
                    )],
                )
                .await;
            assert_ne!(successful.suggestions[0].status, "error");

            let oversized = workspace.document("broken/.npmrc", &"x".repeat(1024 * 1024 + 1), 2);
            assert!(
                session
                    .registry_context(&oversized, ManifestKind::NpmPackageJson)
                    .failure_message()
                    .is_some()
            );
            let healthy = npm_context(&session, &input);
            assert!(healthy.failure_message().is_none());
            assert!(registry_url(&healthy, "demo").starts_with("https://cached.test/"));
            let resolved = session.resolve_document(input).await;
            assert_ne!(resolved.suggestions[0].status, "error");
        }

        #[test]
        fn three_hundred_member_registry_contexts_fit_in_one_snapshot() {
            let workspace = TestWorkspace::new("registry-many-members");
            let session = session();

            for index in 0..300 {
                let input = npm_input(&workspace, &format!("packages/member-{index}/package.json"));
                assert!(
                    npm_context(&session, &input).failure_message().is_none(),
                    "registry context {index} exceeded the shared snapshot budget"
                );
            }
        }

        #[tokio::test]
        async fn local_dependency_stays_fresh_when_registry_configuration_fails() {
            let (workspace, consumer) = local_member_workspace("local-registry-failure");
            let config = std::fs::File::create(workspace.root.join(".npmrc")).unwrap();
            config.set_len(1024 * 1024 + 1).unwrap();
            let session = crate::test_support::test_session(false);
            let local_dependency = local_dependency(&session, &consumer);
            assert_eq!(local_dependency.name, "local-package");
            assert_eq!(local_dependency.requirement, "^1.0.0");
            assert_eq!(local_dependency.ecosystem, Ecosystem::Npm);
            let graph = session.workspace_graph(&consumer);
            assert!(
                graph
                    .resolve(&dependency("local-package", "^1.0.0"))
                    .is_some()
            );
            assert!(graph.resolve(&local_dependency).is_some());

            let output = session.resolve_document(consumer.clone()).await;

            let suggestion = output
                .suggestions
                .iter()
                .find(|suggestion| suggestion.dependency.name == "local-package")
                .unwrap();
            assert_ne!(suggestion.status, "error");
            assert_eq!(suggestion.latest.as_deref(), Some("1.0.0"));
            assert!(npm_context(&session, &consumer).failure_message().is_some());
            assert!(session.document_is_fresh(&consumer));
        }
    }
}
