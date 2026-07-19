use crate::document::test_support::extract_range;
use std::fs::read_to_string;
use std::path::PathBuf;

use super::{DocumentInput, parse_document};
use versionlens_model::Ecosystem::Composer;

#[test]
fn parses_composer_json_dependency_groups() {
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("parses-composer-json-dependency-groups.json").to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Composer);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[1].group, "require");
    assert_eq!(dependencies[1].name, "php");
    assert_eq!(dependencies[2].name, "phpunit/phpunit");
    assert_eq!(dependencies[3].requirement, "../local");
    assert_eq!(dependencies[4].requirement, "git@example.com:org/repo.git");
    assert_eq!(dependencies[5].group, "require-dev");
}

#[test]
fn parses_composer_package_link_groups() {
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("parses-composer-package-link-groups.json").to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Composer);
    assert_eq!(dependencies[0].group, "conflict");
    assert_eq!(dependencies[0].name, "vendor/bad");
    assert_eq!(dependencies[0].requirement, "<1.0 || >=2.0");
    assert_eq!(dependencies[1].group, "replace");
    assert_eq!(dependencies[1].name, "vendor/original");
    assert_eq!(dependencies[1].requirement, "self.version");
    assert_eq!(dependencies[2].group, "provide");
    assert_eq!(dependencies[2].name, "psr/log-implementation");
    assert_eq!(dependencies[2].requirement, "1.0|2.0");
}

#[test]
fn parses_composer_stability_flags_and_references_as_suffixes() {
    let text =
        package_file_fixture("parses-composer-stability-flags-and-references-as-suffixes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].name, "monolog/monolog");
    assert_eq!(dependencies[0].requirement, "1.0.*");
    assert_eq!(dependencies[0].requirement_suffix, "@beta");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.0.*@beta"
    );
    assert_eq!(dependencies[1].name, "acme/foo");
    assert_eq!(dependencies[1].requirement, "dev-master");
    assert_eq!(dependencies[1].requirement_suffix, "#abc123");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "dev-master#abc123"
    );
    assert_eq!(dependencies[2].name, "acme/bar");
    assert_eq!(dependencies[2].requirement, "1.0.x-dev");
    assert_eq!(dependencies[2].requirement_suffix, "#def456");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "1.0.x-dev#def456"
    );
    assert_eq!(dependencies[3].name, "acme/inline");
    assert_eq!(dependencies[3].requirement, "dev-bugfix");
    assert_eq!(dependencies[3].requirement_suffix, " as 1.0.x-dev");
    assert_eq!(
        extract_range(text, dependencies[3].requirement_range),
        "dev-bugfix as 1.0.x-dev"
    );
}

#[test]
fn parses_smoke_composer_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-composer-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 7);
    assert_eq!(dependencies[0].ecosystem, Composer);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "1.0.0");
    assert_eq!(dependencies[1].group, "require");
    assert_eq!(dependencies[1].name, "php");
    assert_eq!(dependencies[1].requirement, "^7.1.3");
    assert_eq!(dependencies[3].name, "phpunit/phpunit");
    assert_eq!(dependencies[3].requirement, "13.2.1");
    assert_eq!(dependencies[5].group, "require-dev");
    assert_eq!(dependencies[6].name, "squizlabs/php_codesniffer");
    assert_eq!(dependencies[6].requirement, "^4.0.1");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/json_manifest/tests/composer")
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
