use super::{DocumentInput, parse_document_with_dependency_paths};
use crate::document::test_support::extract_range;
use versionlens_model::Ecosystem::Dub;

#[test]
fn parses_dub_json_dependency_groups() {
    let text = package_file_fixture("parses-dub-json-dependency-groups.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/dub.json", "json");

    assert_eq!(dependencies.len(), 5);
    assert_dub_dependency(text, &dependencies, "dependencies", "vibe-d", "~>0.9.7");
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
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/dub.json", "json");

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
        &DocumentInput::new(
            "file:///work/dub.json".to_owned(),
            "json".to_owned(),
            text.to_owned(),
            None,
        ),
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
    let dependencies = crate::support::tests::parse_test_document(
        text,
        "file:///work/dub.selections.json",
        "json",
    );

    assert_eq!(dependencies.len(), 2);
    assert_dub_dependency(text, &dependencies, "versions", "gtk-d:gtkd", "3.11.0");
}

#[test]
fn parses_dub_sdl_dependency_directives() {
    let text = package_file_fixture("parses-dub-sdl-dependency-directives.selections.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/dub.sdl", "plaintext");

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
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/json_manifest/tests/dub",
        name,
    )
}

fn assert_dub_dependency(
    text: &str,
    dependencies: &[versionlens_model::Dependency],
    group: &str,
    name: &str,
    requirement: &str,
) {
    assert_eq!(dependencies[0].ecosystem, Dub);
    assert_eq!(dependencies[0].group, group);
    assert_eq!(dependencies[0].name, name);
    assert_eq!(dependencies[0].requirement, requirement);
    crate::support::tests::assert_dependency_requirement_range(text, dependencies, 0, requirement);
}
