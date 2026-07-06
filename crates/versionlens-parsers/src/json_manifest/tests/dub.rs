use super::{DocumentInput, parse_document, parse_document_with_dependency_paths};
use crate::document::test_support::extract_range;
use crate::model::Ecosystem::Dub;
use std::fs::read_to_string;
use std::path::PathBuf;

#[test]
fn parses_dub_json_dependency_groups() {
    let text = package_file_fixture("parses-dub-json-dependency-groups.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Dub);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "vibe-d");
    assert_eq!(dependencies[0].requirement, "~>0.9.7");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "~>0.9.7"
    );
    assert_eq!(dependencies[1].name, "painlessjson");
    assert_eq!(dependencies[1].requirement, "1.4.0");
    assert_eq!(dependencies[2].name, "local");
    assert_eq!(dependencies[2].requirement, "../local");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "../local"
    );
    assert_eq!(dependencies[3].name, "remote");
    assert_eq!(dependencies[3].requirement, "git@example.com:org/repo.git");
    assert_eq!(
        extract_range(text, dependencies[3].requirement_range),
        "git@example.com:org/repo.git"
    );
    assert_eq!(dependencies[4].group, "versions");
    assert_eq!(dependencies[4].name, "imageformats");
    assert_eq!(dependencies[4].requirement, "1.0.0");
}

#[test]
fn parses_dub_json_configuration_dependencies() {
    let text = package_file_fixture("parses-dub-json-configuration-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Dub);
    assert_eq!(dependencies[0].group, "configurations.tls.dependencies");
    assert_eq!(dependencies[0].name, "openssl");
    assert_eq!(dependencies[0].requirement, "~>2.0.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "~>2.0.0"
    );
    assert_eq!(dependencies[1].group, "configurations.tls.dependencies");
    assert_eq!(dependencies[1].name, "localdep");
    assert_eq!(dependencies[1].requirement, "../localdep");
    assert_eq!(dependencies[1].hosted_url.as_deref(), Some("path"));
}

#[test]
fn parses_configured_dub_subpackages() {
    let text = package_file_fixture("parses-configured-dub-subpackages.txt");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/dub.json".to_owned(),
            language_id: "json".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["dependencies", "versions", "subPackages"],
    );

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "vibe-d");
    assert_eq!(dependencies[0].requirement, "~>0.9.7");
    assert_eq!(dependencies[1].group, "subPackages");
    assert_eq!(dependencies[1].name, "standardpaths");
    assert_eq!(dependencies[1].requirement, "~>0.2.1");
}

#[test]
fn parses_dub_selections_versions() {
    let text = package_file_fixture("parses-dub-selections-versions.json");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.selections.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Dub);
    assert_eq!(dependencies[0].group, "versions");
    assert_eq!(dependencies[0].name, "gtk-d:gtkd");
    assert_eq!(dependencies[0].requirement, "3.11.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "3.11.0"
    );
}

#[test]
fn parses_dub_sdl_dependency_directives() {
    let text = package_file_fixture("parses-dub-sdl-dependency-directives.selections.json");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.sdl".to_owned(),
        language_id: "plaintext".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Dub);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "vibe-d");
    assert_eq!(dependencies[0].requirement, "~>0.9.7");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "~>0.9.7"
    );
    assert_eq!(dependencies[1].name, "localdep");
    assert_eq!(dependencies[1].requirement, "../localdep");
    assert_eq!(dependencies[1].hosted_url.as_deref(), Some("path"));
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "../localdep"
    );
    assert_eq!(dependencies[2].name, "remote");
    assert_eq!(
        dependencies[2].requirement,
        "git+https://example.org/remote.git"
    );
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "git+https://example.org/remote.git"
    );
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/json_manifest/tests/dub")
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
