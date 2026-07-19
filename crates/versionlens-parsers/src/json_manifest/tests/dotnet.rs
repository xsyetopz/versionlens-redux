use super::{DocumentInput, parse_document};
use crate::document::test_support::extract_range;
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_dotnet_project_json_dependencies() {
    let text = package_file_fixture("parses-dotnet-project-json-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/project.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Dotnet);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "Newtonsoft.Json");
    assert_eq!(dependencies[0].requirement, "13.0.1");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "13.0.1"
    );
    assert_eq!(dependencies[1].name, "NUnit");
    assert_eq!(dependencies[1].requirement, "4.3.2");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "4.3.2"
    );
    assert_eq!(dependencies[2].group, "frameworks.net472.dependencies");
    assert_eq!(dependencies[2].name, "System.Text.Json");
    assert_eq!(dependencies[2].requirement, "8.0.5");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "8.0.5"
    );
    assert_eq!(dependencies[3].group, "runtimes.win.dependencies");
    assert_eq!(dependencies[3].name, "runtime.win.System.IO");
    assert_eq!(dependencies[3].requirement, "4.3.0");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/json_manifest/tests/dotnet")
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
