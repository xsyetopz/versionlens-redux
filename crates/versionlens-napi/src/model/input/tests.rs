use super::{NativeApplyCommandInput, NativeDocumentInput};
use std::fs::read_to_string;
use std::path::PathBuf;

#[test]
fn maps_document_workspace_root_to_core() {
    let input = NativeDocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("maps-document-workspace-root-to-core.json").to_owned(),
        workspace_root: Some("/work".to_owned()),
    }
    .into_core();

    assert_eq!(input.workspace_root.as_deref(), Some("/work"));
}

#[test]
fn maps_apply_input_document_to_core() {
    let (document, command, dependency_name, selected_version) = NativeApplyCommandInput {
        document: NativeDocumentInput {
            uri: "file:///work/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("maps-apply-input-document-to-core.json").to_owned(),
            workspace_root: Some("/work".to_owned()),
        },
        command: Some("updateMajor".to_owned()),
        dependency_name: Some("left-pad".to_owned()),
        selected_version: Some("1.2.3+build.2".to_owned()),
    }
    .into_parts();

    assert_eq!(document.uri, "file:///work/package.json");
    assert_eq!(document.workspace_root.as_deref(), Some("/work"));
    assert_eq!(command.as_deref(), Some("updateMajor"));
    assert_eq!(dependency_name.as_deref(), Some("left-pad"));
    assert_eq!(selected_version.as_deref(), Some("1.2.3+build.2"));
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-napi/src/model/input/tests")
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
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
