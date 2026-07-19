use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_smoke_dotnet_project_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-project-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/project.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 12);
    assert_eq!(dependencies[0].group, "Project.Sdk");
    assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(dependencies[1].name, "Version");
    assert_eq!(dependencies[1].requirement, "1.2.3");
    assert_eq!(dependencies[2].name, "AssemblyVersion");
    assert_eq!(dependencies[2].requirement, "");
    assert_eq!(dependencies[3].name, "jQuery");
    assert_eq!(dependencies[3].requirement, "3.7");
    assert_eq!(dependencies[4].requirement, "(5.0,)");
    assert_eq!(dependencies[5].requirement, "[2.22]");
    assert_eq!(dependencies[6].requirement, "(,10.9]");
    assert_eq!(dependencies[7].requirement, "[12,13)");
    assert_eq!(dependencies[9].requirement, "1.*");
    assert_eq!(dependencies[10].name, "AngularJS.Core");
    assert_eq!(dependencies[10].requirement, "1.*");
    assert_eq!(dependencies[11].requirement, "1.0.112.2");
}

#[test]
fn parses_smoke_dotnet_props_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-props-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/default.props".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Dotnet);
    assert_eq!(dependencies[0].group, "PackageReference");
    assert_eq!(dependencies[0].name, "Microsoft.NET.Test.Sdk");
    assert_eq!(dependencies[0].requirement, "15.6.2");
    assert_eq!(dependencies[2].name, "MSTest.TestFramework");
}

#[test]
fn parses_smoke_dotnet_targets_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-targets-smoke-shapes.props");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/default.targets".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "PackageReference");
    assert_eq!(
        dependencies[0].name,
        "Microsoft.Extensions.DependencyInjection.Abstractions"
    );
    assert_eq!(dependencies[0].requirement, "10.0.9");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "10.0.9"
    );
    assert_eq!(
        dependencies[1].name,
        "Microsoft.Extensions.Logging.Abstractions"
    );
}

#[test]
fn parses_smoke_dotnet_versionless_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-versionless-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/project.no-version.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].group, "Project.Sdk");
    assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(dependencies[1].name, "jQuery");
    assert_eq!(dependencies[1].requirement, "*");
    assert_eq!(dependencies[2].name, "Nerdbank.GitVersioning");
    assert_eq!(dependencies[2].requirement, "*");
    assert_eq!(dependencies[3].name, "Microsoft.NET.Test.Sdk");
    assert_eq!(dependencies[3].requirement, "18.7.0");
    assert_eq!(
        extract_range(text, dependencies[3].requirement_range),
        "18.7.0"
    );
}

#[test]
fn parses_smoke_dotnet_auth_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-auth-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/auth.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "Project.Sdk");
    assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(dependencies[1].group, "PackageReference");
    assert_eq!(dependencies[1].name, "Private.VersionLens.Package");
    assert_eq!(dependencies[1].requirement, "*");
    assert_eq!(dependencies[1].requirement_prefix, " Version=\"");
    assert_eq!(dependencies[1].requirement_suffix, "\"");
    assert_eq!(extract_range(text, dependencies[1].requirement_range), "");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/dotnet_xml/tests/project_smoke")
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
