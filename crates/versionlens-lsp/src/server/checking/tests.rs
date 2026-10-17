use anyhow::Result;
use lsp_server::{Connection, Message};
use versionlens_core::{WorkspaceCheckEvent, WorkspaceCheckResult};
use versionlens_model::DocumentInput;

use super::{Analyzed, WorkspaceController};
use crate::protocol::DocumentStatus;
use crate::state::{CheckedDocument, VersionLensLspState, VersionLensTextDocument};

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

    let generation = controller.groups[0].checking.generation();
    controller.handle(
        &state,
        &server,
        (
            0,
            WorkspaceCheckEvent {
                generation,
                result: WorkspaceCheckResult::SnapshotLimit {
                    uri: uris[0].to_owned(),
                    limit: 1,
                },
            },
        ),
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

    let generation = controller.groups[0].checking.generation();
    controller.finish_analysis(
        &state,
        &server,
        Analyzed {
            group: 0,
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

const OPEN: &str = "file:///work/package.json";

/// A state with one open manifest, and its current checking input.
fn open_state() -> (VersionLensLspState, DocumentInput) {
    let mut state = VersionLensLspState::with_workspace(None, Vec::new());
    state.open_document(VersionLensTextDocument {
        uri: OPEN.to_owned(),
        language_id: "json".to_owned(),
        text: "{\"dependencies\":{\"example\":\"1.0.0\"}}".to_owned(),
        workspace_root: None,
    });
    state.set_document_version(OPEN, 1);
    let input = state
        .document_work(OPEN)
        .map(|work| work.input)
        .expect("the document is open");
    (state, input)
}

fn methods(sent: &[(String, serde_json::Value)]) -> Vec<&str> {
    sent.iter().map(|(method, _)| method.as_str()).collect()
}

fn statuses(sent: &[(String, serde_json::Value)]) -> Vec<&str> {
    sent.iter()
        .filter(|(method, _)| method == "versionlens/status")
        .filter_map(|(_, params)| params["status"].as_str())
        .collect()
}

#[test]
fn a_cancelled_check_publishes_nothing() -> Result<()> {
    let (state, input) = open_state();
    let (mut controller, _runtime) = controller(&state)?;
    let (server, client) = Connection::memory();
    controller.set_status(&server, OPEN, DocumentStatus::Busy)?;
    sent(&client);

    let generation = controller.groups[0].checking.generation();
    controller.handle(
        &state,
        &server,
        (
            0,
            WorkspaceCheckEvent {
                generation,
                result: WorkspaceCheckResult::Document {
                    input: Box::new(input),
                    result: Err(versionlens_core::CHECKING_CANCELLED.to_owned()),
                },
            },
        ),
    )?;

    assert_eq!(sent(&client), []);
    Ok(())
}

#[test]
fn a_failed_check_is_logged_and_marks_the_document_in_error() -> Result<()> {
    let (state, input) = open_state();
    let (mut controller, _runtime) = controller(&state)?;
    let (server, client) = Connection::memory();
    controller.set_status(&server, OPEN, DocumentStatus::Busy)?;
    sent(&client);

    let generation = controller.groups[0].checking.generation();
    controller.handle(
        &state,
        &server,
        (
            0,
            WorkspaceCheckEvent {
                generation,
                result: WorkspaceCheckResult::Document {
                    input: Box::new(input),
                    result: Err("the resolver failed".to_owned()),
                },
            },
        ),
    )?;

    let sent = sent(&client);
    assert!(!methods(&sent).contains(&"textDocument/publishDiagnostics"));
    assert_eq!(statuses(&sent), ["error"]);
    assert!(sent.iter().any(|(method, params)| {
        method == "window/logMessage"
            && params["message"]
                .as_str()
                .is_some_and(|message| message.contains("the resolver failed"))
    }));
    Ok(())
}

#[test]
fn snapshot_limits_and_discovery_failures_are_logged_not_published() -> Result<()> {
    let (state, _) = open_state();
    let (mut controller, _runtime) = controller(&state)?;
    let (server, client) = Connection::memory();
    let generation = controller.groups[0].checking.generation();
    let root = std::env::temp_dir().join("versionlens-missing-root");
    for result in [
        WorkspaceCheckResult::SnapshotLimit {
            uri: OPEN.to_owned(),
            limit: 1,
        },
        WorkspaceCheckResult::DiscoveryFailure(versionlens_core::WorkspaceDiscoveryFailure {
            path: Some(root.join("package.json")),
            kind: versionlens_core::WorkspaceDiscoveryFailureKind::InvalidUtf8,
        }),
    ] {
        controller.handle(
            &state,
            &server,
            (0, WorkspaceCheckEvent { generation, result }),
        )?;
    }

    let sent = sent(&client);
    assert_eq!(methods(&sent), ["window/logMessage", "window/logMessage"]);
    Ok(())
}
