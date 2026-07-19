use super::{DocumentInput, parse_document, parse_document_with_dependency_paths};
use crate::document::test_support::extract_range;
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Npm;

#[test]
fn parses_package_json_nested_wildcard_and_scalars() {
    let text = package_file_fixture("parses-package-json-nested-wildcard-and-scalars.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 9);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "1.2.3");
    assert_eq!(dependencies[1].group, "packageManager");
    assert_eq!(dependencies[1].name, "pnpm");
    assert_eq!(dependencies[1].requirement, "9.1.2");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "9.1.2"
    );
    assert_eq!(dependencies[2].name, "react");
    assert_eq!(dependencies[3].name, "@scope/pkg");
    assert_eq!(dependencies[4].group, "overrides");
    assert_eq!(dependencies[4].name, "child");
    assert_eq!(dependencies[5].group, "jspm.dependencies");
    assert_eq!(dependencies[6].group, "pnpm.overrides");
    assert_eq!(dependencies[6].name, "leaf");
    assert_eq!(dependencies[7].group, "workspaces.catalog");
    assert_eq!(dependencies[7].name, "react-dom");
    assert_eq!(dependencies[8].group, "workspaces.catalogs.testing");
    assert_eq!(dependencies[8].name, "jest");
}

#[test]
fn parses_package_json_pnpm_package_extensions_by_default() {
    let text = package_file_fixture("parses-package-json-pnpm-package-extensions-by-default.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(
        dependencies[0].group,
        "pnpm.packageExtensions.react-redux.peerDependencies"
    );
    assert_eq!(dependencies[0].name, "react-dom");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "*");
    assert_eq!(
        dependencies[1].group,
        "pnpm.packageExtensions.vite@5.optionalDependencies"
    );
    assert_eq!(dependencies[1].name, "fsevents");
    assert_eq!(dependencies[1].requirement, "^2.3.3");
}

#[test]
fn parses_package_json_bun_catalogs_by_default() {
    let text = package_file_fixture("parses-package-json-bun-catalogs-by-default.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].group, "catalog");
    assert_eq!(dependencies[0].name, "typescript");
    assert_eq!(dependencies[1].group, "catalogs.build");
    assert_eq!(dependencies[1].name, "webpack");
    assert_eq!(dependencies[2].group, "workspaces.catalog");
    assert_eq!(dependencies[2].name, "react");
    assert_eq!(dependencies[3].name, "react-dom");
    assert_eq!(dependencies[4].group, "workspaces.catalogs.testing");
    assert_eq!(dependencies[4].name, "jest");
}

#[test]
fn parses_configured_smoke_npm_custom_dependency_paths() {
    let text = package_file_fixture("parses-configured-smoke-npm-custom-dependency-paths.txt");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["customDependencies".to_owned()],
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "customDependencies");
    assert_eq!(dependencies[0].name, "@types/hammerjs");
    assert_eq!(dependencies[0].requirement, "2.0.33");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "2.0.33"
    );
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/json_manifest/tests/npm_nested")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    <PathBuf as From<&str>>::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
