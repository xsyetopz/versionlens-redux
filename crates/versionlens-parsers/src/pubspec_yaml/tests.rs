use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document, parse_document_with_dependency_paths};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Pub;

#[test]
fn parses_pubspec_yaml_dependencies() {
    let text = package_file_fixture("parses-pubspec-yaml-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].ecosystem, Pub);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[0].requirement, "1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.2.3"
    );
    assert_eq!(dependencies[1].group, "dependencies");
    assert_eq!(dependencies[1].name, "http");
    assert_eq!(dependencies[1].requirement, "^1.2.0");
    assert_eq!(dependencies[2].requirement, "*");
    assert_eq!(dependencies[3].name, "local");
    assert_eq!(dependencies[3].requirement, "./local");
    assert_eq!(dependencies[4].name, "repo");
    assert_eq!(dependencies[4].requirement, "git@example.test/repo.git");
    assert_eq!(dependencies[5].name, "hosted_dep");
    assert_eq!(dependencies[5].requirement, "1.0.0");
    assert_eq!(
        dependencies[5].hosted_url.as_deref(),
        Some("https://pub.example.test")
    );
    assert_eq!(dependencies[5].hosted_name.as_deref(), Some("hosted_alias"));
    assert_eq!(dependencies[6].group, "dev_dependencies");
    assert_eq!(dependencies[6].requirement, "2.0.0");
    assert_eq!(
        extract_range(text, dependencies[6].requirement_range),
        "2.0.0"
    );
    assert_eq!(dependencies[7].group, "dependency_overrides");
    assert_eq!(dependencies[7].name, "override_dep");
}

#[test]
fn parses_pubspec_git_tag_pattern_dependencies_as_git_source() {
    let text =
        package_file_fixture("parses-pubspec-git-tag-pattern-dependencies-as-git-source.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "kittens");
    assert_eq!(
        dependencies[0].requirement,
        "git@github.com:munificent/kittens.git"
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "git@github.com:munificent/kittens.git"
    );
}

#[test]
fn parses_pubspec_yaml_blank_versions() {
    let text = package_file_fixture("parses-pubspec-yaml-blank-versions.yaml");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "http");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(dependencies[0].requirement_suffix, " ");
    assert_eq!(dependencies[1].name, "equatable");
    assert_eq!(dependencies[1].requirement, "");
    assert_eq!(dependencies[1].requirement_prefix, " ");
}

#[test]
fn parses_configured_pubspec_member_dependency_paths() {
    let text = package_file_fixture("parses-configured-pubspec-member-dependency-paths.yaml");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &[
            "dependencies.http".to_owned(),
            "dev_dependencies.*".to_owned(),
        ],
    );

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "http");
    assert_eq!(dependencies[1].group, "dev_dependencies");
    assert_eq!(dependencies[1].name, "test");
}

#[test]
fn ignores_configured_pubspec_array_dependency_paths() {
    let text = package_file_fixture("ignores-configured-pubspec-array-dependency-paths.yaml");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["fonts".to_owned()],
    );

    assert!(dependencies.is_empty());
}

#[test]
fn parses_smoke_pubspec_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-pubspec-smoke-shapes.yaml");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 20);
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[1].name, "flutter");
    assert_eq!(dependencies[1].requirement, "sdk:flutter");
    assert_eq!(dependencies[2].name, "firebase_app_check");
    assert_eq!(dependencies[6].name, "sqflite");
    assert_eq!(
        dependencies[6].requirement,
        "https://github.com/tekartik/sqflite"
    );
    assert_eq!(dependencies[9].name, "glob");
    assert_eq!(dependencies[9].requirement, "2.1.3");
    assert_eq!(dependencies[10].name, "dio");
    assert_eq!(dependencies[10].requirement, "1.*");
    assert_eq!(
        dependencies[10].hosted_url.as_deref(),
        Some("https://pub.dev/")
    );
    assert_eq!(dependencies[11].name, "http_parser");
    assert_eq!(dependencies[11].requirement, "../../");
    assert_eq!(dependencies[12].group, "dev_dependencies");
    assert_eq!(dependencies[12].name, "flutter_test");
    assert_eq!(dependencies[12].requirement, "sdk:flutter");
    assert_eq!(dependencies[13].name, "build_test");
    assert_eq!(dependencies[16].group, "dependency_overrides");
    assert_eq!(dependencies[19].name, "mobx_codegen");
    assert_eq!(dependencies[19].requirement, "*");
}

#[test]
fn parses_hosted_pub_dependency_without_version_with_insert_range() {
    let text =
        package_file_fixture("parses-hosted-pub-dependency-without-version-with-insert-range.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "hosted_dep");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://pub.example.test")
    );
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("hosted_alias"));
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[0].requirement_prefix, "\n    version: ");
}

#[test]
fn parses_pubspec_overrides_dependency_overrides_only() {
    let text = package_file_fixture("parses-pubspec-overrides-dependency-overrides-only.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec_overrides.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "dependency_overrides");
    assert_eq!(dependencies[0].name, "local_override");
    assert_eq!(dependencies[0].requirement, "../local_override");
    assert_eq!(dependencies[1].group, "dependency_overrides");
    assert_eq!(dependencies[1].name, "hosted_override");
    assert_eq!(dependencies[1].requirement, "^2.0.0");
}

#[test]
fn parses_pubspec_sdk_dependencies_as_non_registry_specs() {
    let text = package_file_fixture("parses-pubspec-sdk-dependencies-as-non-registry-specs.yaml");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "flutter");
    assert_eq!(dependencies[0].requirement, "sdk:flutter");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "flutter"
    );
    assert_eq!(dependencies[1].group, "dev_dependencies");
    assert_eq!(dependencies[1].name, "flutter_test");
    assert_eq!(dependencies[1].requirement, "sdk:flutter");
}

#[test]
fn parses_pubspec_workspace_paths_as_local_dependencies() {
    let text = package_file_fixture("parses-pubspec-workspace-paths-as-local-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "workspace");
    assert_eq!(dependencies[0].name, "packages/shared");
    assert_eq!(dependencies[0].requirement, "packages/shared");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "packages/shared"
    );
    assert_eq!(dependencies[1].group, "workspace");
    assert_eq!(dependencies[1].name, "packages/client");
    assert_eq!(dependencies[1].requirement, "packages/client");
    assert_eq!(dependencies[2].group, "dependencies");
    assert_eq!(dependencies[2].name, "http");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/pubspec_yaml/tests")
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
