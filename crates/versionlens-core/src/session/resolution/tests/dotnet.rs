use super::*;

#[test]
fn dotnet_local_nuget_source_does_not_resolve_versions_from_package_folder() {
    let root = temp_dir().join(format!("versionlens-dotnet-local-{}", id()));
    let source = root.join("packages");
    let package_dir = source.join("newtonsoft.json");
    create_dir_all(package_dir.join("13.0.1")).unwrap();
    create_dir_all(package_dir.join("13.0.3")).unwrap();
    write(
        root.join("NuGet.config"),
        r#"<configuration>
  <packageSources>
    <clear />
    <add key="local" value="./packages" />
  </packageSources>
</configuration>"#,
    )
    .unwrap();

    let output = standard_session().resolve_document(DocumentInput::new(
        format!("file://{}", root.join("app.csproj").display()),
        "xml".to_owned(),
        package_file_fixture(
            "dotnet-local-nuget-source-does-not-resolve-versions-from-package-folder.txt",
        ),
        Some(root.to_string_lossy().into_owned()),
    ));

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "unresolved", None);

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_local_nuget_source_does_not_resolve_flat_nupkg_files() {
    let root = temp_dir().join(format!("versionlens-dotnet-flat-local-{}", id()));
    let source = root.join("packages");
    create_dir_all(&source).unwrap();
    write(source.join("Newtonsoft.Json.13.0.3.nupkg"), "").unwrap();
    write(source.join("Newtonsoft.Json.13.0.1.nupkg"), "").unwrap();
    write(
        root.join("NuGet.config"),
        format!(
            r#"<configuration><packageSources><add key="local" value="file://{}" /></packageSources></configuration>"#,
            source.display()
        ),
    )
    .unwrap();

    let output = standard_session().resolve_document(DocumentInput::new(
        format!("file://{}", root.join("app.csproj").display()),
        "xml".to_owned(),
        package_file_fixture("dotnet-local-nuget-source-does-not-resolve-flat-nupkg-files.txt"),
        Some(root.to_string_lossy().into_owned()),
    ));

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "unresolved", None);

    remove_dir_all(root).unwrap();
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/dotnet", name)
}
