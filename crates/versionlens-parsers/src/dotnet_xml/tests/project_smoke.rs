use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document};

#[test]
fn parses_smoke_dotnet_project_smoke_shapes() {
    let text = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <AssemblyVersion></AssemblyVersion>
    <Version>1.2.3</Version>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="jQuery" Version="3.7" />
    <PackageReference Include="Microsoft.Azure.ServiceBus" Version="(5.0,)" />
    <PackageReference Include="Microsoft.Azure.DocumentDB.Core" Version="[2.22]" />
    <PackageReference Include="Microsoft.Extensions.Logging.Abstractions" Version="(,10.9]" />
    <PackageReference Include="Newtonsoft.Json" Version="[12,13)" />
    <PackageReference Include="NUnit" Version="3.0.0-beta-5" />
    <PackageReference Include="AngularJS.Core" Version="1.*" />
    <PackageReference Update="AngularJS.Core" Version="1.*" />
    <PackageReference Include="System.Data.SQLite" Version="1.0.112.2" />
  </ItemGroup>
</Project>"#;
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
    let text = r#"<Project>
  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk" Version="15.6.2" />
    <PackageReference Include="MSTest.TestAdapter" Version="1.2.0" />
    <PackageReference Include="MSTest.TestFramework" Version="1.2.0" />
  </ItemGroup>
</Project>"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/default.props".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Dotnet);
    assert_eq!(dependencies[0].group, "PackageReference");
    assert_eq!(dependencies[0].name, "Microsoft.NET.Test.Sdk");
    assert_eq!(dependencies[0].requirement, "15.6.2");
    assert_eq!(dependencies[2].name, "MSTest.TestFramework");
}

#[test]
fn parses_smoke_dotnet_targets_smoke_shapes() {
    let text = r#"<Project>
  <ItemGroup>
    <PackageReference
      Include="Microsoft.Extensions.DependencyInjection.Abstractions"
      Version="10.0.9"
    />
    <PackageReference Include="Microsoft.Extensions.Logging.Abstractions" Version="10.0.9" />
  </ItemGroup>
</Project>"#;
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
    let text = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>netcoreapp1.1</TargetFramework>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="jQuery" />
    <PackageReference Include="Nerdbank.GitVersioning" PrivateAssets="all" />
    <PackageReference Include="Microsoft.NET.Test.Sdk">
      <Version>18.7.0</Version>
    </PackageReference>
  </ItemGroup>
</Project>"#;
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
    assert_eq!(dependencies[3].requirement, "*");
    assert_eq!(extract_range(text, dependencies[3].requirement_range), "");
}

#[test]
fn parses_smoke_dotnet_auth_smoke_shapes() {
    let text = r#"<Project Sdk="Microsoft.NET.Sdk">
  <ItemGroup>
    <PackageReference Include="Private.VersionLens.Package" />
  </ItemGroup>
</Project>"#;
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
