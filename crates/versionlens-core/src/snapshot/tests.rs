use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::DocumentInput;
use versionlens_parsers::parse_document;

use super::dependency_signature;

#[test]
fn dependency_signature_ignores_workspace_and_catalog_specs() {
    let left = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("dependency-signature-ignores-workspace-and-catalog-specs.json"),
        workspace_root: None,
    });
    let right = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "dependency-signature-ignores-workspace-and-catalog-specs-2.json",
        ),
        workspace_root: None,
    });

    assert_eq!(dependency_signature(&left), dependency_signature(&right));
}

#[test]
fn dependency_signature_ignores_npm_package_manager_changes() {
    let left = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("dependency-signature-ignores-npm-package-manager-changes.json"),
        workspace_root: None,
    });
    let right = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "dependency-signature-ignores-npm-package-manager-changes-2.json",
        ),
        workspace_root: None,
    });

    assert_eq!(dependency_signature(&left), dependency_signature(&right));
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/core/snapshot/tests")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
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
