use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document};

#[test]
fn parses_dotnet_xml_dependencies() {
    let text = r#"<Project Sdk="Microsoft.NET.Sdk/8.0.100">
  <PropertyGroup>
    <Version>1.2.3</Version>
    <AssemblyVersion></AssemblyVersion>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
    <PackageReference Update="Serilog" VersionOverride="3.1.0" />
    <PackageReference Include="NoVersionAttribute" />
    <PackageVersion Include="Microsoft.Extensions.Logging" Version="8.0.0" />
    <DotNetCliToolReference Include="dotnet-ef" Version="8.0.1" />
  </ItemGroup>
</Project>"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Dotnet);
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
fn parses_dotnet_xml_non_empty_versionless_package_reference() {
    let text = r#"<Project>
  <ItemGroup>
    <PackageReference Include="ChildVersionNoAttribute"></PackageReference>
  </ItemGroup>
</Project>"#;
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
fn dotnet_ignores_package_reference_child_version() {
    let text = r#"<Project>
  <ItemGroup>
    <PackageReference Include="ChildVersionNoAttribute">
      <Version>18.7.0</Version>
    </PackageReference>
  </ItemGroup>
</Project>"#;
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
    assert_eq!(
        extract_range(text, dependencies[0].range),
        "ChildVersionNoAttribute"
    );
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
}

#[test]
fn dotnet_invalid_xml_returns_no_dependencies() {
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3" /></Project>"#.to_owned(),
        workspace_root: None,
    });

    assert!(dependencies.is_empty());
}

#[test]
fn dotnet_dependency_order_follows_dependency_properties() {
    let text = r#"<Project>
  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
    <PackageVersion Include="Central.Package" Version="1.0.0" />
    <GlobalPackageReference Include="Global.Package" Version="2.0.0" />
    <DotNetCliToolReference Include="dotnet-ef" Version="8.0.1" />
  </ItemGroup>
  <Sdk Name="Microsoft.Build.CentralPackageVersions" Version="2.1.3" />
</Project>"#;
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
    let text = r#"<Project Sdk="Microsoft.NET.Sdk/8.0.100"></Project>"#;
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
    let text = r#"<Project>
  <ItemGroup>
    <PackageReference include = 'Newtonsoft.Json' version = "13.0.3" />
  </ItemGroup>
</Project>"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Dotnet);
    assert_eq!(dependencies[0].name, "Newtonsoft.Json");
    assert_eq!(dependencies[0].requirement, "13.0.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "13.0.3"
    );
}
