use crate::document::test_support::extract_range;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_dotnet_project_json_dependencies() {
    let text = package_file_fixture("parses-dotnet-project-json-dependencies.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/project.json", "json");

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
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/json_manifest/tests/dotnet",
        name,
    )
}
