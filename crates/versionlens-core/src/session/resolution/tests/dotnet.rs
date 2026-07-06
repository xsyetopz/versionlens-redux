use super::{DocumentInput, standard_session};
use std::env;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;

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

    let output = standard_session().resolve_document(DocumentInput {
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "dotnet-local-nuget-source-does-not-resolve-versions-from-package-folder.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    });

    assert_eq!(output.suggestions[0].status, "unresolved");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());

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

    let output = standard_session().resolve_document(DocumentInput {
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "dotnet-local-nuget-source-does-not-resolve-flat-nupkg-files.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    });

    assert_eq!(output.suggestions[0].status, "unresolved");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());

    remove_dir_all(root).unwrap();
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/dotnet")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
