use super::{DocumentInput, Ecosystem, session_with_properties};

#[test]
fn maven_dependency_properties_filter_before_extraction() {
    let session = session_with_properties(Ecosystem::Maven, &["project.parent"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: r#"
<project>
  <parent>
    <groupId>org.springframework.boot</groupId>
    <artifactId>spring-boot-starter-parent</artifactId>
    <version>3.3.0</version>
  </parent>
  <dependencies>
    <dependency>
      <groupId>junit</groupId>
      <artifactId>junit</artifactId>
      <version>4.13.2</version>
    </dependency>
  </dependencies>
</project>
"#
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(
        output.dependencies[0].name,
        "org.springframework.boot:spring-boot-starter-parent"
    );
}

#[test]
fn dotnet_project_sdk_matches_dependency_properties() {
    let session = session_with_properties(Ecosystem::Dotnet, &["Project.Sdk"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: r#"<Project><Sdk Name="Microsoft.NET.Sdk" Version="8.0.100" /></Project>"#.to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "Microsoft.NET.Sdk");
}

#[test]
fn dotnet_dependency_properties_filter_before_extraction() {
    let session =
        session_with_properties(Ecosystem::Dotnet, &["Project.ItemGroup.PackageReference"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: r#"
<Project>
  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
    <PackageVersion Include="Serilog" Version="3.1.1" />
  </ItemGroup>
</Project>
"#
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "Newtonsoft.Json");
}

#[test]
fn dotnet_project_json_dependency_properties_match_json_paths() {
    let session = session_with_properties(Ecosystem::Dotnet, &["frameworks.*.dependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///project.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{
  "dependencies": {
    "Newtonsoft.Json": "13.0.1"
  },
  "frameworks": {
    "net472": {
      "dependencies": {
        "System.Text.Json": "8.0.5"
      }
    }
  }
}"#
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "System.Text.Json");
    assert_eq!(
        output.dependencies[0].group,
        "frameworks.net472.dependencies"
    );
}
