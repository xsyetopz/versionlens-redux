use super::parse_paket_source_urls;
use crate::document::test_support::{extract_range, parse_fixture};
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_paket_dependencies_nuget_lines() {
    let text = package_file_fixture("parses-paket-dependencies-nuget-lines.txt");
    let dependencies = parse_fixture(text, "file:///work/paket.dependencies", "plaintext");

    assert_eq!(dependencies.len(), 3);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Dotnet,
            "paket.dependencies",
            "Newtonsoft.Json",
            "13.0.3",
        ),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "13.0.3"
    );
    assert_eq!(dependencies[1].name, "FSharp.Core");
    assert_eq!(dependencies[1].requirement, ">= 8.0");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        ">= 8.0"
    );
    assert_eq!(dependencies[2].name, "Paket.Core");
    assert_eq!(dependencies[2].requirement, "");
    assert_eq!(extract_range(text, dependencies[2].requirement_range), "");
}

#[test]
fn parses_paket_references_as_unresolved_project_references_without_update_ranges() {
    let text = package_file_fixture(
        "parses-paket-references-as-unresolved-project-references-without-update-ranges.txt",
    );
    let dependencies = parse_fixture(text, "file:///work/paket.references", "plaintext");

    assert_eq!(dependencies.len(), 2);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Dotnet,
            "paket.references",
            "Newtonsoft.Json",
            "",
        ),
    );
    crate::support::tests::assert_requirement_range_ends_at_dependency(&dependencies, 0);
}

#[test]
fn parses_paket_source_urls() {
    assert_eq!(
        parse_paket_source_urls("source https://api.nuget.org/v3/index.json\nsource ./local\n"),
        ["https://api.nuget.org/v3/index.json"]
    );
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture("tests/fixtures/versionlens-parsers/src/paket/tests", name)
}
