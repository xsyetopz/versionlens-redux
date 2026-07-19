use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_smoke_dotnet_fsproj_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-fsproj-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/project.fsproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].group, "Project.Sdk");
    assert_eq!(dependencies[0].name, "FSharp.NET.Sdk");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(dependencies[1].group, "Project.Sdk");
    assert_eq!(dependencies[1].name, "Microsoft.NET.Sdk");
    assert_eq!(dependencies[1].requirement, "*");
    assert_eq!(dependencies[2].name, "FSharp.Core");
    assert_eq!(dependencies[2].requirement, "4.1.2");
    assert_eq!(dependencies[3].name, "FSharp.Net.Sdk");
    assert_eq!(dependencies[3].requirement, "1.0.1");
}

#[test]
fn parses_smoke_dotnet_override_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-override-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/project.override.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "Project.Sdk");
    assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(dependencies[1].group, "PackageReference");
    assert_eq!(dependencies[1].name, "jQuery");
    assert_eq!(dependencies[1].requirement, "3.7.*");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "3.7.*"
    );
}

#[test]
fn parses_smoke_dotnet_central_package_props_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-central-package-props-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Directory.Packages.props".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "Sdk");
    assert_eq!(
        dependencies[0].name,
        "Microsoft.Build.CentralPackageVersions"
    );
    assert_eq!(dependencies[0].requirement, "2.1.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "2.1.3"
    );
    assert_eq!(dependencies[1].group, "GlobalPackageReference");
    assert_eq!(dependencies[1].name, "Microsoft.Azure.ServiceBus");
    assert_eq!(dependencies[1].requirement, "(3.0,)");
    assert_eq!(dependencies[2].group, "PackageVersion");
    assert_eq!(dependencies[2].name, "System.Text.Json");
    assert_eq!(dependencies[2].requirement, "4.7.2");
}

#[test]
fn parses_smoke_dotnet_bom_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-bom-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/project.bom.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Dotnet);
    assert_eq!(dependencies[0].group, "Project.Sdk");
    assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(dependencies[1].name, "jQuery");
    assert_eq!(dependencies[1].requirement, "3.7.1");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "3.7.1"
    );
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/dotnet_xml/tests/sdk_smoke")
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
