use crate::document::test_support::{extract_range, parse_fixture};
use crate::{DocumentInput, parse_document_with_dependency_paths};
use versionlens_model::Ecosystem::Pub;

#[test]
fn parses_pubspec_yaml_dependencies() {
    let text = package_file_fixture("pubspec-yaml-dependencies.txt");
    let dependencies = parse_fixture(text, "file:///work/pubspec.yaml", "yaml");

    assert_eq!(dependencies.len(), 8);
    crate::support::tests::assert_dependency_with_range(
        text,
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Pub, "version", "version", "1.2.3"),
        "1.2.3",
    );
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(1, Pub, "dependencies", "http", "^1.2.0"),
    );
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
    let text = package_file_fixture("pubspec-git-tag-pattern-dependencies-as-git-source.txt");
    let dependencies = parse_fixture(text, "file:///work/pubspec.yaml", "yaml");

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
    let text = package_file_fixture("yaml-blank-versions.yaml");
    let dependencies = parse_fixture(text, "file:///work/pubspec.yaml", "yaml");

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
    let text = package_file_fixture("configured-pubspec-member-dependency-paths.yaml");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput::new(
            "file:///work/pubspec.yaml".to_owned(),
            "yaml".to_owned(),
            text.to_owned(),
            None,
        ),
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
        &DocumentInput::new(
            "file:///work/pubspec.yaml".to_owned(),
            "yaml".to_owned(),
            text.to_owned(),
            None,
        ),
        &["fonts".to_owned()],
    );

    assert!(dependencies.is_empty());
}

#[test]
fn parses_smoke_pubspec_smoke_shapes() {
    let text = package_file_fixture("smoke-pubspec-smoke-shapes.yaml");
    let dependencies = parse_fixture(text, "file:///work/pubspec.yaml", "yaml");

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
    let text = package_file_fixture("hosted-pub-dependency-without-version-with-insert-range.txt");
    let dependencies = parse_fixture(text, "file:///work/pubspec.yaml", "yaml");

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
    let text = package_file_fixture("pubspec-overrides-dependency-overrides-only.txt");
    let dependencies = parse_fixture(text, "file:///work/pubspec_overrides.yaml", "yaml");

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
    let text = package_file_fixture("sdk-dependencies-as-non-registry-specs.yaml");
    let dependencies = parse_fixture(text, "file:///work/pubspec.yaml", "yaml");

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
    let text = package_file_fixture("pubspec-workspace-paths-as-local-dependencies.txt");
    let dependencies = parse_fixture(text, "file:///work/pubspec.yaml", "yaml");

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
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/pubspec_yaml/tests",
        name,
    )
}
