use super::*;
use support::{TestWorkspace, dependency};
use versionlens_model::Position;

pub(crate) mod support;

fn root_input(root: &Path) -> DocumentInput {
    DocumentInput::new(
        root.join("package.json").to_string_lossy(),
        "json",
        fs::read_to_string(root.join("package.json")).unwrap(),
        Some(root.to_string_lossy().into()),
    )
}

fn workspace_graph(input: &DocumentInput) -> WorkspaceGraph {
    let Some(root) = workspace_root(input) else {
        return WorkspaceGraph::default();
    };
    let documents = document_path(input)
        .map(|current| (current, (input.text.clone(), input.version)))
        .into_iter()
        .collect::<WorkspaceDocuments>();
    WorkspaceGraph::for_workspace(&root, &documents).bind_source(input)
}

fn coordinated_plan(
    graph: &WorkspaceGraph,
    input: &DocumentInput,
) -> versionlens_model::WorkspaceEditPlan {
    graph
        .coordinated_plan(input, &[], "a", "2.0.0")
        .unwrap()
        .unwrap()
}

#[test]
fn discovers_stale_open_member() {
    let workspace = TestWorkspace::npm();
    let root = workspace.path();
    let input = DocumentInput::new(
        root.join("packages/a/package.json").to_string_lossy(),
        "json",
        r#"{"name":"a","version":"2.0.0"}"#,
        Some(root.to_string_lossy().into()),
    );
    let graph = workspace_graph(&input);
    assert_eq!(graph.members.len(), 1);
    assert_eq!(resolved_version(&graph, "a"), "2.0.0");
}

#[test]
fn duplicate_is_unresolved() {
    let workspace = TestWorkspace::npm();
    let root = workspace.path();
    workspace.write_all(&[
        (
            "packages/a/package.json",
            r#"{"name":"dup","version":"1.0.0"}"#,
        ),
        (
            "packages/b/package.json",
            r#"{"name":"dup","version":"2.0.0"}"#,
        ),
    ]);
    let input = root_input(&root);
    let graph = workspace_graph(&input);
    assert!(graph.resolve(&dependency("dup", "workspace:*")).is_none());
}

#[test]
fn open_root_manifest_controls_membership_and_package_manager_policy() {
    let workspace = TestWorkspace::npm();
    let root = workspace.path();
    let mut input = root_input(root);
    input.text = r#"{"workspaces":[],"packageManager":"pnpm@10.0.0"}"#.into();
    let graph = workspace_graph(&input);
    assert!(graph.members.is_empty());
    input.text = r#"{"workspaces":["packages/*"],"packageManager":"pnpm@10.0.0"}"#.into();
    let graph = workspace_graph(&input);
    assert_eq!(resolved_version(&graph, "a"), "1.0.0");
    assert!(graph.resolve(&dependency("a", "^1.0.0")).is_none());
}

#[test]
fn recognizes_lerna_fixed_and_independent_policy_without_overriding_manifest_versions() {
    let workspace = TestWorkspace::npm();
    let root = workspace.path();
    fs::write(
        root.join("packages/a/package.json"),
        r#"{"name":"a","version":"1.2.3"}"#,
    )
    .unwrap();
    fs::write(root.join("lerna.json"), r#"{"version":"independent"}"#).unwrap();
    let input = root_input(&root);
    let graph = workspace_graph(&input);
    assert_eq!(graph.policy, WorkspacePolicy::Independent);
    assert_eq!(resolved_version(&graph, "a"), "1.2.3");
    fs::write(root.join("lerna.json"), r#"{"version":"3.0.0"}"#).unwrap();
    assert_eq!(workspace_graph(&input).policy, WorkspacePolicy::Fixed);
}

fn resolved_version(graph: &WorkspaceGraph, name: &str) -> String {
    graph
        .resolve(&dependency(name, "workspace:*"))
        .expect("workspace dependency should resolve")
        .version
}

#[test]
fn cargo_members_inherit_the_root_workspace_version() {
    let workspace = TestWorkspace::new("cargo-inherited-version");
    workspace.write_all(&[
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\"]\n[workspace.package]\nversion = \"1.2.3\"\n",
        ),
        (
            "crates/a/Cargo.toml",
            "[package]\nname = \"a\"\nversion.workspace = true\n",
        ),
        (
            "crates/b/Cargo.toml",
            "[package]\nname = \"b\"\nversion = { workspace = true }\n",
        ),
    ]);
    let input = workspace.document(
        "crates/b/Cargo.toml",
        "[package]\nname = \"b\"\nversion = { workspace = true }\n",
        1,
    );
    let graph = workspace_graph(&input);
    for (name, relative) in [("a", "../a"), ("b", "../b")] {
        let mut dependency = dependency(name, relative);
        dependency.ecosystem = Ecosystem::Cargo;
        dependency.hosted_url = Some("path".to_owned());

        let resolved = graph
            .resolve(&dependency)
            .expect("inherited Cargo member must resolve locally");
        assert_eq!(resolved.version, "1.2.3");
        assert!(
            resolved
                .manifest
                .ends_with(format!("crates/{name}/Cargo.toml"))
        );
    }
}

#[test]
fn coordinated_edits_use_the_open_manifest_snapshot() {
    let workspace = TestWorkspace::npm();
    let root = workspace.path();
    let mut input = DocumentInput::new(
        root.join("packages/a/package.json").to_string_lossy(),
        "json",
        "{\n  \"name\": \"a\",\n  \"version\": \"1.0.0\"\n}",
        Some(root.to_string_lossy().into()),
    );
    input.version = Some(7);
    let graph = workspace_graph(&input);
    let plan = coordinated_plan(&graph, &input);
    assert_eq!(plan.documents.len(), 1);
    let document = &plan.documents[0];
    assert_eq!(document.document.version, Some(7));
    assert_eq!(document.document.text_hash, document_text_hash(&input.text));
    assert_eq!(document.edits.len(), 1);
    assert_eq!(
        document.edits[0].range.start,
        Position {
            line: 2,
            character: 14
        }
    );
    assert_eq!(document.edits[0].new_text, "2.0.0");
}

#[test]
fn coordinated_edits_keep_member_text_and_hash_from_one_snapshot() {
    let workspace = TestWorkspace::npm();
    let root = workspace.path();
    let input = root_input(root);
    let graph = workspace_graph(&input);
    let member = root.join("packages/a/package.json");
    let original = fs::read_to_string(&member).unwrap();
    fs::write(&member, "{\n\"name\":\"a\",\"version\":\"9.0.0\"}").unwrap();
    let plan = coordinated_plan(&graph, &input);
    let document = plan
        .documents
        .iter()
        .find(|document| document.document.uri.ends_with("packages/a/package.json"))
        .unwrap();
    assert_eq!(document.document.text_hash, document_text_hash(&original));
    assert_ne!(
        document.document.text_hash,
        document_text_hash(&fs::read_to_string(member).unwrap())
    );
    assert_eq!(document.edits[0].range.start.line, 0);
}
