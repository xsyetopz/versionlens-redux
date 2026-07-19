use lsp_types::{TextDocumentSyncCapability, TextDocumentSyncKind, Uri, WorkspaceFolder};

use super::{DISPLAY_CODE_LENS_COMMAND, VersionLensLspState, VersionLensTextDocument};

#[test]
fn capabilities_match_implemented_document_and_command_protocol() -> Result<(), &'static str> {
    let capabilities = VersionLensLspState::server_capabilities();
    let Some(TextDocumentSyncCapability::Options(sync)) = capabilities.text_document_sync else {
        return Err("expected explicit text document sync options");
    };
    assert_eq!(sync.open_close, Some(true));
    assert_eq!(sync.change, Some(TextDocumentSyncKind::FULL));
    let Some(code_lens) = capabilities.code_lens_provider else {
        return Err("expected CodeLens capabilities");
    };
    assert_eq!(code_lens.resolve_provider, Some(false));
    let Some(execute_command) = capabilities.execute_command_provider else {
        return Err("expected execute command capabilities");
    };
    assert_eq!(execute_command.commands, [DISPLAY_CODE_LENS_COMMAND]);
    Ok(())
}

#[test]
fn selects_the_deepest_workspace_folder_and_decodes_its_path() -> Result<(), String> {
    let root_uri = uri("file:///workspace")?;
    let workspace_folders = vec![
        folder("file:///workspace/project", "project")?,
        folder("file:///workspace/project/nested%20folder", "nested")?,
    ];
    let mut state = VersionLensLspState::with_workspace(Some(root_uri), workspace_folders);
    let document_uri = "file:///workspace/project/nested%20folder/package.json";

    state.open_document(document(document_uri, None));

    assert_eq!(
        state
            .documents
            .get(document_uri)
            .and_then(|document| document.workspace_root.as_deref()),
        Some("/workspace/project/nested folder")
    );
    Ok(())
}

#[test]
fn root_uri_is_used_only_for_documents_inside_it() -> Result<(), String> {
    let mut state =
        VersionLensLspState::with_workspace(Some(uri("file:///workspace")?), Vec::new());
    let inside = "file:///workspace/package.json";
    let outside = "file:///workspace-other/package.json";

    state.open_document(document(inside, None));
    state.open_document(document(outside, None));

    assert_eq!(
        state
            .documents
            .get(inside)
            .and_then(|document| document.workspace_root.as_deref()),
        Some("/workspace")
    );
    assert_eq!(
        state
            .documents
            .get(outside)
            .and_then(|document| document.workspace_root.as_deref()),
        None
    );
    Ok(())
}

#[test]
fn code_lenses_are_visible_informational_commands_and_close_with_the_document() {
    let mut state = VersionLensLspState::standard();
    let uri = "file:///workspace/package.json";
    state.open_document(VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"version":"1.0.0"}"#.to_owned(),
        workspace_root: Some("/workspace".to_owned()),
    });

    let lenses = state.code_lenses(uri);
    assert!(!lenses.is_empty());
    assert!(lenses.iter().all(|lens| {
        lens.command.as_ref().is_some_and(|command| {
            !command.title.is_empty()
                && command.command == DISPLAY_CODE_LENS_COMMAND
                && command.arguments.is_none()
        })
    }));

    state.close_document(uri);
    assert!(state.code_lenses(uri).is_empty());
    assert!(!state.documents.contains_key(uri));
}

#[test]
fn changes_unknown_document_without_diagnostics() {
    let mut state = VersionLensLspState::standard();
    assert!(
        state
            .change_document("file:///missing/package.json", "{}".to_owned())
            .is_empty()
    );
}

fn uri(value: &str) -> Result<Uri, String> {
    value.parse::<Uri>().map_err(|error| error.to_string())
}

fn folder(value: &str, name: &str) -> Result<WorkspaceFolder, String> {
    Ok(WorkspaceFolder {
        uri: uri(value)?,
        name: name.to_owned(),
    })
}

fn document(uri: &str, workspace_root: Option<String>) -> VersionLensTextDocument {
    VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "json".to_owned(),
        text: "{}".to_owned(),
        workspace_root,
    }
}
