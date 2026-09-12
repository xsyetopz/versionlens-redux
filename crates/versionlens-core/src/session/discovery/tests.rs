use std::fs::{self, create_dir_all, write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use versionlens_model::{DocumentInput, Ecosystem, ManifestKind};

use super::{
    WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryFileSize, WorkspaceDiscoveryIoOperation,
    WorkspaceDiscoveryLimits, WorkspaceDiscoveryOptions,
};
use crate::{EnabledProviderConfig, FilePatternConfig, ProviderSettings, VersionLensSession};

static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(name: &str) -> Self {
        let id = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "versionlens-discovery-{name}-{}-{id}",
            std::process::id()
        ));
        create_dir_all(&root).unwrap();
        Self { root }
    }

    fn write(&self, relative: &str, text: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(relative);
        create_dir_all(path.parent().unwrap()).unwrap();
        write(&path, text).unwrap();
        path
    }

    fn uri(&self, relative: &str) -> String {
        crate::workspace_file_uri(&self.root.canonicalize().unwrap().join(relative)).unwrap()
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn session() -> VersionLensSession {
    crate::support::tests::test_session(false)
}

fn options(workspace: &TestWorkspace) -> WorkspaceDiscoveryOptions {
    WorkspaceDiscoveryOptions::new(vec![workspace.root.clone()])
}

fn successful_uris(
    results: &[Result<DocumentInput, super::WorkspaceDiscoveryFailure>],
) -> Vec<String> {
    let mut uris = results
        .iter()
        .filter_map(|result| result.as_ref().ok().map(|input| input.uri.clone()))
        .collect::<Vec<_>>();
    uris.sort();
    uris
}

#[test]
fn discovers_unopened_nested_and_runtime_manifests_incrementally() {
    let workspace = TestWorkspace::new("nested");
    workspace.write("packages/a/package.json", r#"{"dependencies":{}}"#);
    workspace.write("tools/.nvmrc", "22\n");
    workspace.write("notes/readme.txt", "not a manifest");

    let results = session()
        .discover_workspace_documents(options(&workspace))
        .collect::<Vec<_>>();

    assert!(results.iter().all(Result::is_ok));
    assert_eq!(
        successful_uris(&results),
        [
            workspace.uri("packages/a/package.json"),
            workspace.uri("tools/.nvmrc")
        ]
    );
}

#[test]
fn exclusions_prune_disk_but_keep_explicit_overlays() {
    let workspace = TestWorkspace::new("exclusions");
    workspace.write("apps/included/package.json", "{}");
    workspace.write("apps/excluded/package.json", "{}");
    let mut request = options(&workspace);
    request.exclusions = vec!["**/excluded/**".to_owned()];
    request.overlays.push(DocumentInput::new(
        workspace.uri("apps/excluded/new/package.json"),
        "json",
        r#"{"dependencies":{"dirty":"1"}}"#,
        None,
    ));

    let results = session()
        .discover_workspace_documents(request)
        .collect::<Vec<_>>();

    assert_eq!(
        successful_uris(&results),
        [
            workspace.uri("apps/excluded/new/package.json"),
            workspace.uri("apps/included/package.json")
        ]
    );
}

#[test]
fn disabled_provider_is_rejected_before_the_file_size_or_read_gate() {
    let workspace = TestWorkspace::new("disabled");
    workspace.write("Cargo.toml", vec![b'x'; 512]);
    workspace.write("package.json", "{}");
    let mut config = crate::support::tests::session_config(crate::default(), false);
    config.enabled_providers = vec![EnabledProviderConfig {
        ecosystem: Ecosystem::Npm,
        manifest_kind: Some(ManifestKind::NpmPackageJson),
    }];
    let session = VersionLensSession::new(config);
    let mut request = options(&workspace);
    request.limits.max_file_size = 8;

    let results = session
        .discover_workspace_documents(request)
        .collect::<Vec<_>>();

    assert!(results.iter().all(Result::is_ok));
    assert_eq!(successful_uris(&results), [workspace.uri("package.json")]);
}

#[test]
fn provider_exclusions_apply_to_disk_without_hiding_supported_overlays() {
    let workspace = TestWorkspace::new("provider-exclusions");
    workspace.write("obj/generated.csproj", "<Project />");
    workspace.write("obj/package.json", "{}");
    workspace.write("src/application.csproj", "<Project />");
    let mut request = options(&workspace);
    request
        .provider_exclusions
        .push(super::WorkspaceProviderExclusion {
            ecosystem: Ecosystem::Dotnet,
            patterns: vec!["**/obj/**".to_owned()],
        });
    request.overlays = vec![
        DocumentInput::new(
            workspace.uri("obj/dirty.csproj"),
            "xml",
            "<Project />",
            None,
        ),
        DocumentInput::new(workspace.uri("obj/new/package.json"), "json", "{}", None),
    ];
    let results = session()
        .discover_workspace_documents(request)
        .collect::<Vec<_>>();
    assert!(results.iter().all(Result::is_ok));
    assert_eq!(
        successful_uris(&results),
        [
            workspace.uri("obj/new/package.json"),
            workspace.uri("obj/package.json"),
            workspace.uri("src/application.csproj"),
        ]
    );
}

#[test]
fn git_ignores_nested_rules_and_repository_excludes_but_keep_hidden_manifests() {
    let workspace = TestWorkspace::new("git-ignore");
    workspace.write(".git/HEAD", "ref: refs/heads/main\n");
    workspace.write(".git/info/exclude", "info/package.json\n");
    workspace.write(".gitignore", "ignored/\n");
    workspace.write("nested/.gitignore", "*.json\n!package.json\n");
    workspace.write("ignored/package.json", "{}");
    workspace.write("info/package.json", "{}");
    workspace.write("nested/package.json", "{}");
    workspace.write("nested/other.json", "{}");
    workspace.write("tools/.nvmrc", "22\n");

    let results = session()
        .discover_workspace_documents(options(&workspace))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut uris = results
        .iter()
        .map(|input| input.uri.clone())
        .collect::<Vec<_>>();
    uris.sort();
    assert_eq!(
        uris,
        [
            workspace.uri("nested/package.json"),
            workspace.uri("tools/.nvmrc")
        ]
    );
}

#[cfg(unix)]
#[test]
fn internal_directory_symlinks_are_never_followed() {
    use std::os::unix::fs::symlink;

    let workspace = TestWorkspace::new("internal-symlink");
    workspace.write("real/package.json", "{}");
    symlink(workspace.root.join("real"), workspace.root.join("linked")).unwrap();

    let results = session()
        .discover_workspace_documents(options(&workspace))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(
        results
            .iter()
            .map(|input| input.uri.as_str())
            .collect::<Vec<_>>(),
        [workspace.uri("real/package.json")]
    );
}

#[test]
fn dirty_and_new_overlays_replace_disk_and_preserve_versions() {
    let workspace = TestWorkspace::new("overlays");
    workspace.write("package.json", r#"{"version":"disk"}"#);
    let mut request = options(&workspace);
    request.overlays = vec![
        DocumentInput::new(
            workspace.uri("package.json"),
            "json",
            r#"{"version":"dirty"}"#,
            None,
        )
        .with_version(7),
        DocumentInput::new(
            workspace.uri("packages/new/package.json"),
            "json",
            r#"{"version":"new"}"#,
            None,
        )
        .with_version(1),
    ];

    let results = session()
        .discover_workspace_documents(request)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].text, r#"{"version":"dirty"}"#);
    assert_eq!(results[0].version, Some(7));
    assert_eq!(results[1].text, r#"{"version":"new"}"#);
    assert_eq!(results[1].version, Some(1));
    assert!(results.iter().all(|input| input.workspace_root.is_some()));
}

#[test]
fn configured_patterns_use_escaped_workspace_file_uris() {
    let workspace = TestWorkspace::new("escaped uri");
    let path = workspace.write("ci files/dépendances #1.custom", "{}");
    let config = crate::support::tests::session_config(
        ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ManifestKind::NpmPackageJson,
                pattern: "ci files/dépendances #?.custom".to_owned(),
            }],
            ..crate::default()
        },
        false,
    );

    let input = VersionLensSession::new(config)
        .discover_workspace_documents(options(&workspace))
        .next()
        .unwrap()
        .unwrap();

    assert_eq!(
        input.uri,
        crate::workspace_file_uri(&path.canonicalize().unwrap()).unwrap()
    );
    assert!(input.uri.contains("%20"));
    assert!(input.uri.contains("%C3%A9"));
    assert!(input.uri.contains("%23"));
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_escape_the_workspace() {
    use std::os::unix::fs::symlink;

    let workspace = TestWorkspace::new("symlink-root");
    let outside = TestWorkspace::new("symlink-outside");
    outside.write("package.json", "{}");
    symlink(&outside.root, workspace.root.join("linked")).unwrap();
    workspace.write("package.json", "{}");

    let results = session()
        .discover_workspace_documents(options(&workspace))
        .collect::<Vec<_>>();

    assert_eq!(successful_uris(&results), [workspace.uri("package.json")]);
    assert!(results.iter().any(|result| matches!(
        result,
        Err(failure) if failure.kind == WorkspaceDiscoveryFailureKind::EscapesWorkspace
    )));
}

#[test]
fn invalid_utf8_and_oversize_failures_do_not_hide_later_documents() {
    let workspace = TestWorkspace::new("recover");
    workspace.write("bad/package.json", [0xff, 0xfe]);
    workspace.write("large/Cargo.toml", vec![b'x'; 64]);
    workspace.write("good/.nvmrc", "22");
    let mut request = options(&workspace);
    request.limits.max_file_size = 16;

    let results = session()
        .discover_workspace_documents(request)
        .collect::<Vec<_>>();

    assert_eq!(successful_uris(&results), [workspace.uri("good/.nvmrc")]);
    assert!(results.iter().any(|result| matches!(
        result,
        Err(failure) if failure.kind == WorkspaceDiscoveryFailureKind::InvalidUtf8
    )));
    assert!(results.iter().any(|result| matches!(
        result,
        Err(failure)
            if failure.kind == WorkspaceDiscoveryFailureKind::FileTooLarge(Box::new(
                WorkspaceDiscoveryFileSize { size: 64, limit: 16 }
            ))
    )));
}

#[test]
fn depth_and_file_count_bounds_are_explicit() {
    let workspace = TestWorkspace::new("bounds");
    workspace.write("one/package.json", "{}");
    workspace.write("two/package.json", "{}");
    workspace.write("deep/nested/package.json", "{}");
    let mut depth_request = options(&workspace);
    depth_request.limits = WorkspaceDiscoveryLimits {
        max_files: 10,
        max_visited_entries: 100,
        max_depth: 1,
        max_file_size: 1024,
    };

    let depth_results = session()
        .discover_workspace_documents(depth_request)
        .collect::<Vec<_>>();

    assert!(depth_results.iter().any(|result| matches!(
        result,
        Err(failure)
            if matches!(failure.kind, WorkspaceDiscoveryFailureKind::DepthLimitExceeded { limit: 1 })
    )));
    let mut count_request = options(&workspace);
    count_request.limits.max_files = 1;
    let count_results = session()
        .discover_workspace_documents(count_request)
        .collect::<Vec<_>>();
    assert_eq!(successful_uris(&count_results).len(), 1);
    assert!(count_results.iter().any(|result| matches!(
        result,
        Err(failure)
            if matches!(failure.kind, WorkspaceDiscoveryFailureKind::FileLimitExceeded { limit: 1 })
    )));
}

#[test]
fn sparse_trees_stop_at_the_visited_entry_bound() {
    let workspace = TestWorkspace::new("entry-bound");
    workspace.write("a.txt", "unsupported");
    workspace.write("b.txt", "unsupported");
    workspace.write("c/package.json", "{}");
    let mut request = options(&workspace);
    request.limits.max_visited_entries = 1;

    let results = session()
        .discover_workspace_documents(request)
        .collect::<Vec<_>>();

    assert!(results.iter().any(|result| matches!(
        result,
        Err(failure)
            if failure.kind
                == WorkspaceDiscoveryFailureKind::EntryLimitExceeded { limit: 1 }
    )));
    assert!(results.len() <= 2);
}

#[test]
fn cancellation_is_one_terminal_outcome() {
    let workspace = TestWorkspace::new("cancel");
    workspace.write("package.json", "{}");
    let request = options(&workspace);
    let cancellation = request.cancellation.clone();
    cancellation.cancel();
    let mut discovery = session().discover_workspace_documents(request);

    assert!(matches!(
        discovery.next(),
        Some(Err(failure)) if failure.kind == WorkspaceDiscoveryFailureKind::Cancelled
    ));
    assert!(discovery.next().is_none());
}

#[test]
fn missing_roots_are_reported_with_the_failed_operation() {
    let workspace = TestWorkspace::new("missing-root");
    let missing = workspace.root.join("missing");
    let results = session()
        .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![missing.clone()]))
        .collect::<Vec<_>>();

    assert!(matches!(
        results.as_slice(),
        [Err(failure)]
            if failure.path.as_deref() == Some(Path::new(&missing))
                && matches!(
                    failure.kind,
                    WorkspaceDiscoveryFailureKind::Io {
                        operation: WorkspaceDiscoveryIoOperation::ResolvePath,
                        ..
                    }
                )
    ));
}
