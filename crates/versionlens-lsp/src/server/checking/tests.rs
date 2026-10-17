use anyhow::Result;
use lsp_server::{Connection, Message};
use versionlens_core::{WorkspaceCheckEvent, WorkspaceCheckResult};
use versionlens_model::DocumentInput;

use super::{Analyzed, WorkspaceController};
use crate::protocol::DocumentStatus;
use crate::state::{CheckedDocument, VersionLensLspState};

fn controller(
    state: &VersionLensLspState,
) -> Result<(WorkspaceController, tokio::runtime::Runtime)> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let controller = WorkspaceController::start(state, runtime.handle().clone())?;
    Ok((controller, runtime))
}

/// The messages the controller has sent so far.
fn sent(client: &Connection) -> Vec<(String, serde_json::Value)> {
    client
        .receiver
        .try_iter()
        .filter_map(|message| match message {
            Message::Notification(notification) => Some((notification.method, notification.params)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_snapshot_limit_ends_every_busy_document_in_error() -> Result<()> {
    let state = VersionLensLspState::with_workspace(None, Vec::new());
    let (mut controller, _runtime) = controller(&state)?;
    let (server, client) = Connection::memory();
    let uris = ["file:///a/package.json", "file:///b/package.json"];
    for uri in uris {
        controller.set_status(&server, uri, DocumentStatus::Busy)?;
    }
    sent(&client);

    let generation = controller.checking.generation();
    controller.handle(
        &state,
        &server,
        WorkspaceCheckEvent {
            generation,
            result: WorkspaceCheckResult::SnapshotLimit {
                uri: uris[0].to_owned(),
                limit: 1,
            },
        },
    )?;

    let mut errors = sent(&client)
        .into_iter()
        .filter(|(method, params)| method == "versionlens/status" && params["status"] == "error")
        .map(|(_, params)| params["uri"].as_str().unwrap_or_default().to_owned())
        .collect::<Vec<_>>();
    errors.sort();
    assert_eq!(errors, uris);
    Ok(())
}

#[test]
fn an_analysis_of_a_closed_document_is_not_published() -> Result<()> {
    let state = VersionLensLspState::with_workspace(None, Vec::new());
    let (mut controller, _runtime) = controller(&state)?;
    let (server, client) = Connection::memory();
    let uri = "file:///closed/package.json";
    let mut input = DocumentInput::new(uri.to_owned(), "json".to_owned(), "{}".to_owned(), None);
    input.version = Some(1);

    let generation = controller.checking.generation();
    controller.finish_analysis(
        &state,
        &server,
        Analyzed {
            generation,
            document: CheckedDocument {
                uri: uri.to_owned(),
                version: Some(1),
                generation: 0,
                input,
                open: true,
            },
            analysis: Err("unsaved text".to_owned()),
        },
    )?;

    assert_eq!(sent(&client), []);
    Ok(())
}
