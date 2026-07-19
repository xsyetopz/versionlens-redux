use super::{DocumentInput, parse_document};
use crate::document::test_support::extract_range;
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Npm;

#[test]
fn parses_smoke_npm_range_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-npm-range-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "@faker-js/faker");
    assert_eq!(dependencies[0].requirement, "> 10.0.0 < 10.5.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "> 10.0.0 < 10.5.0"
    );
    assert_eq!(dependencies[1].name, "typescript");
    assert_eq!(dependencies[1].requirement, "> 6.0.0 < 6.0.3");
}

#[test]
fn parses_smoke_jspm_package_json_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-jspm-package-json-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].group, "jspm.dependencies");
    assert_eq!(dependencies[0].name, "bluebird");
    assert_eq!(dependencies[0].requirement, "^3.7.2");
    assert_eq!(dependencies[1].requirement, "*");
    assert_eq!(dependencies[2].name, "twbs/bootstrap");
    assert_eq!(dependencies[2].requirement, "v5.3.8");
    assert_eq!(dependencies[2].requirement_prefix, "github:twbs/bootstrap#");
    assert_eq!(dependencies[5].group, "jspm.devDependencies");
    assert_eq!(dependencies[5].name, "core-js");
    assert_eq!(dependencies[5].requirement, "^3.49.0");
}

#[test]
fn parses_smoke_typical_package_json_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-typical-package-json-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 14);
    assert_eq!(dependencies[1].name, "typescript");
    assert_eq!(dependencies[1].requirement, "6.0.3");
    assert_eq!(dependencies[2].name, "twbs/bootstrap");
    assert_eq!(dependencies[2].requirement, "v5.3.8");
    assert_eq!(dependencies[3].name, "expressjs/express");
    assert_eq!(dependencies[3].requirement, "v5.2.1");
    assert_eq!(
        dependencies[3].requirement_prefix,
        "expressjs/express#semver:"
    );
    assert_eq!(dependencies[6].name, "@types/node");
    assert_eq!(dependencies[6].requirement, "latest");
    assert_eq!(dependencies[12].name, "invalid");
    assert_eq!(dependencies[12].requirement, ">=4.5.");
    assert!(!dependencies.iter().any(|dependency| {
        dependency.group == "customDependencies" && dependency.name == "@types/hammerjs"
    }));
}

#[test]
fn parses_smoke_npm_workspaces_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-npm-workspaces-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].group, "workspaces.catalog");
    assert_eq!(dependencies[0].name, "react");
    assert_eq!(dependencies[1].group, "workspaces.catalog");
    assert_eq!(dependencies[1].name, "react-dom");
    assert_eq!(dependencies[2].group, "workspaces.catalogs.testing");
    assert_eq!(dependencies[2].name, "jest");
    assert_eq!(dependencies[3].group, "workspaces.catalogs.testing");
    assert_eq!(dependencies[3].name, "testing-library");
}

#[test]
fn parses_smoke_npm_overrides_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-npm-overrides-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "overrides");
    assert_eq!(dependencies[0].name, "semver");
    assert_eq!(dependencies[0].requirement, "7.8.5");
    assert_eq!(dependencies[1].group, "overrides");
    assert_eq!(dependencies[1].name, "typescript");
    assert_eq!(dependencies[2].name, "semver");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/json_manifest/tests/npm_smoke")
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
