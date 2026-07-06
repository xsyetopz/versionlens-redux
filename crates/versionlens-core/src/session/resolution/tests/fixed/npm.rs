use super::{DocumentInput, RegistryResponseInput, standard_session};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::Ecosystem::Npm;

#[test]
fn npm_bare_relative_paths_are_invalid_versions() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("npm-bare-relative-paths-are-invalid-versions.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "filepackage".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "error");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("invalid version")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_github_url_without_ref_is_fixed_git_dependency() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("npm-github-url-without-ref-is-fixed-git-dependency.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/url".to_owned(),
            ecosystem: Npm,
            body: r#"[{"sha":"abcdef1234567890"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_package_manager_dependencies_ignore_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "npm-package-manager-dependencies-ignore-registry-updates.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "pnpm".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"10.34.4"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("9.1.2"));
    assert!(output.edits.is_empty());
}

#[test]
fn npm_dev_engines_package_manager_dependencies_ignore_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "npm-dev-engines-package-manager-dependencies-ignore-registry-updates.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "npm".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"11.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("^10.0.0"));
    assert!(output.edits.is_empty());
}

#[test]
fn npm_override_reference_dependencies_ignore_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "npm-override-reference-dependencies-ignore-registry-updates.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "bar".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("override reference")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_portal_dependencies_resolve_as_local_paths_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "npm-portal-dependencies-resolve-as-local-paths-without-registry-updates.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directoryNotFound");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("../local"));
    assert!(output.edits.is_empty());
}

#[test]
fn npm_exec_dependencies_are_not_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("npm-exec-dependencies-are-not-registry-updates.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "generated".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "notSupported");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());
}

#[test]
fn npm_patch_dependencies_are_not_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("npm-patch-dependencies-are-not-registry-updates.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "@types/react".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"19.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "notSupported");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/fixed/npm")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
