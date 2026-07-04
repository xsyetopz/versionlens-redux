use super::{NativeApplyCommandInput, NativeDocumentInput};

#[test]
fn maps_document_workspace_root_to_core() {
    let input = NativeDocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: "{}".to_owned(),
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
            text: "{}".to_owned(),
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
