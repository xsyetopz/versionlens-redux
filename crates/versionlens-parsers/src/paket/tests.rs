use super::parse_paket_source_urls;
use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_paket_dependencies_nuget_lines() {
    let text = package_file_fixture("parses-paket-dependencies-nuget-lines.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/paket.dependencies".to_owned(),
        language_id: "plaintext".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Dotnet);
    assert_eq!(dependencies[0].group, "paket.dependencies");
    assert_eq!(dependencies[0].name, "Newtonsoft.Json");
    assert_eq!(dependencies[0].requirement, "13.0.3");
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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/paket.references".to_owned(),
        language_id: "plaintext".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Dotnet);
    assert_eq!(dependencies[0].group, "paket.references");
    assert_eq!(dependencies[0].name, "Newtonsoft.Json");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(
        dependencies[0].requirement_range.start,
        dependencies[0].range.end
    );
    assert_eq!(
        dependencies[0].requirement_range.end,
        dependencies[0].range.end
    );
}

#[test]
fn parses_paket_source_urls() {
    assert_eq!(
        parse_paket_source_urls("source https://api.nuget.org/v3/index.json\nsource ./local\n"),
        ["https://api.nuget.org/v3/index.json"]
    );
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/paket/tests")
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
