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

#[test]
fn parses_dotnet_local_tool_manifests() {
    let text = r#"{
  "version": 1,
  "isRoot": true,
  "tools": {
    "dotnet-ef": { "version": "8.0.4", "commands": ["dotnet-ef"] },
    "csharpier": { "version": "0.28.2", "commands": ["dotnet-csharpier"] }
  }
}
"#;
    for uri in [
        "file:///work/dotnet-tools.json",
        "file:///work/.config/dotnet-tools.json",
    ] {
        let dependencies = crate::support::tests::parse_test_document(text, uri, "json");

        let found: Vec<_> = dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.ecosystem,
                    dependency.group.as_str(),
                    dependency.name.as_str(),
                    extract_range(text, dependency.requirement_range),
                )
            })
            .collect();
        assert_eq!(
            found,
            [
                (Dotnet, "tools", "dotnet-ef", "8.0.4"),
                (Dotnet, "tools", "csharpier", "0.28.2"),
            ],
            "{uri}"
        );
    }
}
