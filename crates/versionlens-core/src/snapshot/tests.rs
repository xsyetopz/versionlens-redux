use versionlens_model::DocumentInput;
use versionlens_parsers::parse_document;

use super::dependency_signature;

#[test]
fn dependency_signature_ignores_workspace_and_catalog_specs() {
    let left = parse_document(&DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("signature-ignores-workspace-and-catalog-specs.json"),
        None,
    ));
    let right = parse_document(&DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("signature-ignores-workspace-and-catalog-range-change.json"),
        None,
    ));

    assert_eq!(dependency_signature(&left), dependency_signature(&right));
}

#[test]
fn dependency_signature_ignores_npm_package_manager_changes() {
    let left = parse_document(&DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("signature-ignores-npm-package-manager-changes.json"),
        None,
    ));
    let right = parse_document(&DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("signature-ignores-npm-package-manager-version-change.json"),
        None,
    ));

    assert_eq!(dependency_signature(&left), dependency_signature(&right));
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/core-scenarios/snapshot/tests", name)
}
