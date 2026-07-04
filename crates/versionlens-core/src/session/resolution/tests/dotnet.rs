use super::{DocumentInput, standard_session};

#[test]
fn dotnet_local_nuget_source_does_not_resolve_versions_from_package_folder() {
    let root =
        std::env::temp_dir().join(format!("versionlens-dotnet-local-{}", std::process::id()));
    let source = root.join("packages");
    let package_dir = source.join("newtonsoft.json");
    std::fs::create_dir_all(package_dir.join("13.0.1")).unwrap();
    std::fs::create_dir_all(package_dir.join("13.0.3")).unwrap();
    std::fs::write(
        root.join("NuGet.config"),
        r#"<configuration>
  <packageSources>
    <clear />
    <add key="local" value="./packages" />
  </packageSources>
</configuration>"#,
    )
    .unwrap();

    let output = standard_session().resolve_document(DocumentInput {
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.1" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    });

    assert_eq!(output.suggestions[0].status, "unresolved");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_local_nuget_source_does_not_resolve_flat_nupkg_files() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-dotnet-flat-local-{}",
        std::process::id()
    ));
    let source = root.join("packages");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("Newtonsoft.Json.13.0.3.nupkg"), "").unwrap();
    std::fs::write(source.join("Newtonsoft.Json.13.0.1.nupkg"), "").unwrap();
    std::fs::write(
        root.join("NuGet.config"),
        format!(
            r#"<configuration><packageSources><add key="local" value="file://{}" /></packageSources></configuration>"#,
            source.display()
        ),
    )
    .unwrap();

    let output = standard_session().resolve_document(DocumentInput {
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.1" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    });

    assert_eq!(output.suggestions[0].status, "unresolved");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());

    std::fs::remove_dir_all(root).unwrap();
}
