use super::*;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use versionlens_model::{Dependency, Position, Range};

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn temp() -> TestWorkspace {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let process = std::process::id();
    loop {
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!("versionlens-workspace-{process}-{serial}"));
        if fs::create_dir(&path).is_ok() {
            fs::create_dir(path.join("packages")).unwrap();
            fs::create_dir(path.join("packages/a")).unwrap();
            return TestWorkspace { root: path };
        }
    }
}

fn dependency(name: &str, requirement: &str) -> Dependency {
    Dependency {
        name: name.into(),
        requirement: requirement.into(),
        ecosystem: Ecosystem::Npm,
        group: "dependencies".into(),
        hosted_url: None,
        hosted_name: None,
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        },
        requirement_range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        },
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: None,
    }
}

fn write_workspace(root: &Path) {
    fs::write(
        root.join("package.json"),
        r#"{"workspaces":["packages/*"]}"#,
    )
    .unwrap();
    fs::write(
        root.join("packages/a/package.json"),
        r#"{"name":"a","version":"1.0.0"}"#,
    )
    .unwrap();
}

fn root_input(root: &Path) -> DocumentInput {
    DocumentInput::new(
        root.join("package.json").to_string_lossy(),
        "json",
        fs::read_to_string(root.join("package.json")).unwrap(),
        Some(root.to_string_lossy().into()),
    )
}

#[test]
fn discovers_stale_open_member() {
    let workspace = temp();
    let root = workspace.path();
    write_workspace(root);
    let input = DocumentInput::new(
        root.join("packages/a/package.json").to_string_lossy(),
        "json",
        r#"{"name":"a","version":"2.0.0"}"#,
        Some(root.to_string_lossy().into()),
    );
    let graph = WorkspaceGraph::for_document(&input);
    assert_eq!(graph.members.len(), 1);
    assert_eq!(resolved_version(&graph, "a"), "2.0.0");
}

#[test]
fn duplicate_is_unresolved() {
    let workspace = temp();
    let root = workspace.path();
    fs::create_dir_all(root.join("packages/b")).unwrap();
    write_workspace(&root);
    fs::write(
        root.join("packages/a/package.json"),
        r#"{"name":"dup","version":"1.0.0"}"#,
    )
    .unwrap();
    fs::write(
        root.join("packages/b/package.json"),
        r#"{"name":"dup","version":"2.0.0"}"#,
    )
    .unwrap();
    let input = root_input(&root);
    let graph = WorkspaceGraph::for_document(&input);
    assert!(graph.resolve(&dependency("dup", "workspace:*")).is_none());
}

#[test]
fn recognizes_lerna_fixed_and_independent_policy_without_overriding_manifest_versions() {
    let workspace = temp();
    let root = workspace.path();
    write_workspace(root);
    fs::write(
        root.join("packages/a/package.json"),
        r#"{"name":"a","version":"1.2.3"}"#,
    )
    .unwrap();
    fs::write(root.join("lerna.json"), r#"{"version":"independent"}"#).unwrap();
    let input = root_input(&root);
    let graph = WorkspaceGraph::for_document(&input);
    assert_eq!(graph.policy, WorkspacePolicy::Independent);
    assert_eq!(resolved_version(&graph, "a"), "1.2.3");
    fs::write(root.join("lerna.json"), r#"{"version":"3.0.0"}"#).unwrap();
    assert_eq!(
        WorkspaceGraph::for_document(&input).policy,
        WorkspacePolicy::Fixed
    );
}

fn resolved_version(graph: &WorkspaceGraph, name: &str) -> String {
    graph
        .resolve(&dependency(name, "workspace:*"))
        .expect("workspace dependency should resolve")
        .version
}
