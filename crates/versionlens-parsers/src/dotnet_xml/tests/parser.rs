use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_dotnet_xml_dependencies() {
    let text = package_file_fixture("parses-dotnet-xml-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].ecosystem, Dotnet);
    assert_eq!(dependencies[0].group, "Project.Sdk");
    assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
    assert_eq!(dependencies[0].requirement, "8.0.100");
    assert_eq!(dependencies[1].group, "PropertyGroup");
    assert_eq!(dependencies[1].name, "Version");
    assert_eq!(dependencies[1].requirement, "1.2.3");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "1.2.3"
    );
    assert_eq!(dependencies[2].group, "PropertyGroup");
    assert_eq!(dependencies[2].name, "AssemblyVersion");
    assert_eq!(dependencies[2].requirement, "");
    assert_eq!(extract_range(text, dependencies[2].requirement_range), "");
    assert_eq!(dependencies[3].group, "PackageReference");
    assert_eq!(dependencies[3].name, "Newtonsoft.Json");
    assert_eq!(dependencies[3].requirement, "13.0.3");
    assert_eq!(
        extract_range(text, dependencies[3].range),
        r#"<PackageReference Include="Newtonsoft.Json" Version="13.0.3" />"#
    );
    assert_eq!(
        extract_range(text, dependencies[3].requirement_range),
        "13.0.3"
    );
    assert_eq!(dependencies[4].name, "Serilog");
    assert_eq!(dependencies[4].requirement, "3.1.0");
    assert_eq!(dependencies[5].name, "NoVersionAttribute");
    assert_eq!(dependencies[5].requirement, "*");
    assert_eq!(dependencies[5].requirement_prefix, " Version=\"");
    assert_eq!(dependencies[5].requirement_suffix, "\"");
    assert_eq!(extract_range(text, dependencies[5].requirement_range), "");
    assert_eq!(dependencies[6].group, "PackageVersion");
    assert_eq!(dependencies[6].name, "Microsoft.Extensions.Logging");
    assert_eq!(dependencies[7].group, "DotNetCliToolReference");
    assert_eq!(dependencies[7].name, "dotnet-ef");
    assert_eq!(dependencies[7].requirement, "8.0.1");
}

#[test]
fn parses_packages_config_dependencies() {
    let text = package_file_fixture("parses-packages-config-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/packages.config".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Dotnet);
    assert_eq!(dependencies[0].group, "packages.package");
    assert_eq!(dependencies[0].name, "jQuery");
    assert_eq!(dependencies[0].requirement, "3.1.1");
    assert_eq!(extract_range(text, dependencies[0].range), "jQuery");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "3.1.1"
    );
    assert_eq!(dependencies[1].group, "packages.package");
    assert_eq!(dependencies[1].name, "NLog");
    assert_eq!(dependencies[1].requirement, "4.3.10");
}

#[test]
fn parses_dotnet_xml_non_empty_versionless_package_reference() {
    let text =
        package_file_fixture("parses-dotnet-xml-non-empty-versionless-package-reference.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "ChildVersionNoAttribute");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(dependencies[0].requirement_prefix, " Version=\"");
    assert_eq!(dependencies[0].requirement_suffix, "\"");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
}

#[test]
fn parses_package_reference_child_version() {
    let text = package_file_fixture("parses-package-reference-child-version.csproj");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "ChildVersionNoAttribute");
    assert_eq!(dependencies[0].requirement, "18.7.0");
    assert_eq!(dependencies[0].requirement_prefix, "");
    assert_eq!(dependencies[0].requirement_suffix, "");
    assert_eq!(
        extract_range(text, dependencies[0].range),
        "ChildVersionNoAttribute"
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "18.7.0"
    );
}

#[test]
fn dotnet_invalid_xml_returns_no_dependencies() {
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: package_file_fixture("dotnet-invalid-xml-returns-no-dependencies.csproj").to_owned(),
        workspace_root: None,
    });

    assert!(dependencies.is_empty());
}

#[test]
fn dotnet_dependency_order_follows_dependency_properties() {
    let text = package_file_fixture("dotnet-dependency-order-follows-dependency-properties.csproj");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    let names = dependencies
        .iter()
        .map(|dependency| dependency.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "Microsoft.Build.CentralPackageVersions",
            "Global.Package",
            "Newtonsoft.Json",
            "Central.Package",
            "dotnet-ef"
        ]
    );
}

#[test]
fn dotnet_project_sdk_attribute_is_parsed_like_upstream() {
    let text = package_file_fixture("dotnet-project-sdk-attribute-is-parsed-like-upstream.csproj");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "Project.Sdk");
    assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
    assert_eq!(dependencies[0].requirement, "8.0.100");
    assert_eq!(
        extract_range(text, dependencies[0].range),
        "Microsoft.NET.Sdk"
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "8.0.100"
    );
}

#[test]
fn parses_dotnet_xml_attributes_case_insensitively() {
    let text = package_file_fixture("parses-dotnet-xml-attributes-case-insensitively.csproj");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Dotnet);
    assert_eq!(dependencies[0].name, "Newtonsoft.Json");
    assert_eq!(dependencies[0].requirement, "13.0.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "13.0.3"
    );
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/dotnet_xml/tests/current")
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
