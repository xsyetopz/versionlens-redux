use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document};

#[test]
fn parses_smoke_dotnet_fsproj_smoke_shapes() {
    let text = r#"<Project Sdk="FSharp.NET.Sdk;Microsoft.NET.Sdk">
  <PropertyGroup>
    <Copyright></Copyright>
    <VersionPrefix>1.0.0</VersionPrefix>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
    <DebugType>portable</DebugType>
    <TargetFramework>netstandard1.6</TargetFramework>
    <OutputType>Library</OutputType>
  </PropertyGroup>
  <ItemGroup>
    <Compile Include="something.fs" />
  </ItemGroup>
  <ItemGroup Condition=" '$(TargetFramework)' == 'netstandard1.6' ">
    <PackageReference Include="FSharp.Core" Version="4.1.2" />
    <PackageReference Include="FSharp.Net.Sdk" Version="1.0.1" PrivateAssets="All" />
  </ItemGroup>
</Project>"#;
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
    let text = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>netcoreapp1.1</TargetFramework>
  </PropertyGroup>

  <ItemGroup>
    <!-- VersionOverride -->
    <PackageReference Include="jQuery" VersionOverride="3.7.*" />
  </ItemGroup>
</Project>"#;
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
    let text = r#"<Project>
  <Sdk Name="Microsoft.Build.CentralPackageVersions" Version="2.1.3" />
  <ItemGroup>
    <PackageVersion Include="System.Text.Json" Version="4.7.2" />
  </ItemGroup>
  <ItemGroup>
    <GlobalPackageReference Include="Microsoft.Azure.ServiceBus" Version="(3.0,)" />
  </ItemGroup>
</Project>"#;
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
    let text = "\u{feff}<Project Sdk=\"Microsoft.NET.Sdk\">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>netcoreapp1.1</TargetFramework>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include=\"jQuery\" VersionOverride=\"3.7.1\" />
  </ItemGroup>
</Project>";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/project.bom.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Dotnet);
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
