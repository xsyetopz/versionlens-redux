use std::collections::HashMap;
use std::time::Instant;

use anyhow::{Context, Result};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
};
use lsp_types::request::{CodeLensRequest, ExecuteCommand, Request as LspRequest};
use lsp_types::{
    CodeLensParams, Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, ExecuteCommandParams, Uri, WorkspaceFolder,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::state::{
    DISPLAY_CODE_LENS_COMMAND, UPDATE_DEPENDENCY_COMMAND, VersionLensLspState,
    VersionLensTextDocument,
};

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeWorkspaceParams {
    #[serde(default)]
    root_uri: Option<Uri>,
    #[serde(default)]
    workspace_folders: Option<Vec<WorkspaceFolder>>,
    #[serde(default)]
    capabilities: lsp_types::ClientCapabilities,
    #[serde(flatten)]
    _remaining: HashMap<String, Value>,
}

mod checking;
mod settings;
mod work;
use checking::WorkspaceController;
use settings::SettingsRequests;
use work::WorkQueue;

pub fn run_stdio_server() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    run_connection(&connection, true)?;
    drop(connection);
    io_threads.join()?;
    Ok(())
}

fn run_connection(connection: &Connection, persistent: bool) -> Result<()> {
    let mut state = initialize(connection, persistent)?;
    let mut settings = SettingsRequests::default();
    let queued = settings.first(connection, &mut state)?;
    run_message_loop(connection, &mut state, settings, queued)
}

fn initialize(connection: &Connection, persistent: bool) -> Result<VersionLensLspState> {
    loop {
        let (id, params) = connection.initialize_start()?;
        let params = match serde_json::from_value::<InitializeWorkspaceParams>(params) {
            Ok(params) => params,
            Err(error) => {
                respond_error(
                    connection,
                    id,
                    ErrorCode::InvalidParams,
                    format!("invalid initialize params: {error}"),
                )?;
                continue;
            }
        };
        let state = VersionLensLspState::with_workspace(
            params.root_uri,
            params.workspace_folders.unwrap_or_default(),
        )
        .with_client(params.capabilities);
        let state = if persistent {
            state.with_application_cache()?
        } else {
            state
        };
        let initialize_result = serde_json::json!({
            "capabilities": VersionLensLspState::server_capabilities(),
        });
        connection.initialize_finish(id, initialize_result)?;
        register_file_watcher(connection, &state)?;
        return Ok(state);
    }
}

fn run_message_loop(
    connection: &Connection,
    state: &mut VersionLensLspState,
    mut settings: SettingsRequests,
    queued: Vec<Message>,
) -> Result<()> {
    let runtime = Runtime::new().context("failed to create LSP async runtime")?;
    let mut work = WorkQueue::new(runtime.handle().clone());
    state.use_workspace_controller();
    let mut checking = WorkspaceController::start(state, runtime.handle().clone())?;
    let mut parts = LoopParts {
        state,
        work: &mut work,
        checking: &mut checking,
        settings: &mut settings,
    };
    let mut stopped = false;
    for message in queued {
        if handle_message(connection, &mut parts, message)? {
            stopped = true;
            break;
        }
    }
    while !stopped {
        let LoopParts {
            state,
            work,
            checking,
            ..
        } = &mut parts;
        let expiry = [work.next_expiry(), checking.next_due()]
            .into_iter()
            .flatten()
            .min()
            .map_or_else(crossbeam_channel::never, crossbeam_channel::at);
        crossbeam_channel::select! {
            recv(expiry) -> now => {
                if let Ok(now) = now {
                    work.expire_applications(connection, now)?;
                    checking.flush_due(state, connection, now)?;
                }
            }
            recv(checking.analyzed) -> analyzed => {
                if let Ok(analyzed) = analyzed { checking.finish_analysis(state, connection, analyzed)?; }
            }
            recv(work.receiver) -> completion => {
                if let Ok(completion) = completion { work.complete(state, connection, completion)?; }
            }
            recv(checking.receiver) -> event => {
                if let Ok(event) = event { checking.handle(state, connection, event)?; }
            }
            recv(connection.receiver) -> message => {
                stopped = match message {
                    Ok(message) => handle_message(connection, &mut parts, message)?,
                    Err(_) => true,
                };
            }
        }
    }
    parts.state.session.cancel_pending_resolutions();
    runtime.shutdown_background();
    Ok(())
}

/// The parts of the server that a client message can change.
struct LoopParts<'a> {
    state: &'a mut VersionLensLspState,
    work: &'a mut WorkQueue,
    checking: &'a mut WorkspaceController,
    settings: &'a mut SettingsRequests,
}

/// Handles one client message. Returns `true` when the server must stop.
fn handle_message(
    connection: &Connection,
    parts: &mut LoopParts<'_>,
    message: Message,
) -> Result<bool> {
    let LoopParts {
        state,
        work,
        checking,
        settings,
    } = parts;
    match message {
        Message::Request(request) => {
            if connection.handle_shutdown(&request)? {
                return Ok(true);
            }
            handle_request(connection, state, work, request)?;
        }
        Message::Notification(notification) => match notification.method.as_str() {
            "exit" => return Ok(true),
            "$/cancelRequest" => {
                if let Some(id) = notification
                    .params
                    .get("id")
                    .and_then(|id| serde_json::from_value::<RequestId>(id.clone()).ok())
                {
                    work.cancel(connection, &id)?;
                }
            }
            "workspace/didChangeConfiguration" => settings.request(connection, state)?,
            _ => {
                handle_notification(connection, state, checking, notification)?;
                work.invalidate(state, connection)?;
            }
        },
        Message::Response(response) if settings::is_answer(&response.id) => {
            settings.answer(connection, state, Some(checking), response)?;
            work.invalidate(state, connection)?;
        }
        Message::Response(response) => work.applied(connection, response)?,
    }
    Ok(false)
}

fn handle_request(
    connection: &Connection,
    state: &VersionLensLspState,
    work: &mut WorkQueue,
    request: Request,
) -> Result<()> {
    let Request { id, method, params } = request;
    match method.as_str() {
        CodeLensRequest::METHOD => {
            let params = match serde_json::from_value::<CodeLensParams>(params) {
                Ok(params) => params,
                Err(error) => {
                    return respond_error(
                        connection,
                        id,
                        ErrorCode::InvalidParams,
                        format!("invalid {} params: {error}", CodeLensRequest::METHOD),
                    );
                }
            };
            work.lenses(state, connection, id, params.text_document.uri.as_str())
        }
        ExecuteCommand::METHOD => {
            let params = match serde_json::from_value::<ExecuteCommandParams>(params) {
                Ok(params) => params,
                Err(error) => {
                    return respond_error(
                        connection,
                        id,
                        ErrorCode::InvalidParams,
                        format!("invalid {} params: {error}", ExecuteCommand::METHOD),
                    );
                }
            };
            match params.command.as_str() {
                DISPLAY_CODE_LENS_COMMAND if params.arguments.is_empty() => {
                    respond(connection, id, Value::Null)
                }
                UPDATE_DEPENDENCY_COMMAND => work.update(state, connection, id, params.arguments),
                _ => respond_error(
                    connection,
                    id,
                    ErrorCode::InvalidParams,
                    format!("unsupported command: {}", params.command),
                ),
            }
        }
        _ => respond_error(
            connection,
            id,
            ErrorCode::MethodNotFound,
            format!("method not found: {method}"),
        ),
    }
}

fn handle_notification(
    connection: &Connection,
    state: &mut VersionLensLspState,
    checking: &mut WorkspaceController,
    notification: Notification,
) -> Result<()> {
    match notification.method.as_str() {
        "workspace/didChangeWorkspaceFolders" => {
            let Ok(params) = serde_json::from_value::<lsp_types::DidChangeWorkspaceFoldersParams>(
                notification.params,
            ) else {
                return Ok(());
            };
            state.change_workspace_folders(params.event);
            checking.replace(state)
        }
        "workspace/didChangeWatchedFiles" => {
            let Ok(params) = serde_json::from_value::<lsp_types::DidChangeWatchedFilesParams>(
                notification.params,
            ) else {
                return Ok(());
            };
            if !params
                .changes
                .iter()
                .any(|change| state.watched_file_is_relevant(change.uri.as_str()))
            {
                return Ok(());
            }
            state.invalidate_workspace();
            checking.replace(state)
        }
        _ => handle_document_notification(connection, state, checking, notification),
    }
}

fn handle_document_notification(
    connection: &Connection,
    state: &mut VersionLensLspState,
    checking: &mut WorkspaceController,
    notification: Notification,
) -> Result<()> {
    match notification.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let Ok(params) =
                serde_json::from_value::<DidOpenTextDocumentParams>(notification.params)
            else {
                return Ok(());
            };
            let uri = params.text_document.uri;
            state.open_document(VersionLensTextDocument {
                uri: uri.to_string(),
                language_id: params.text_document.language_id,
                text: params.text_document.text,
                workspace_root: None,
            });
            state.set_document_version(uri.as_str(), params.text_document.version);
            checking.schedule_document(uri.as_str(), Instant::now(), false);
            Ok(())
        }
        DidChangeTextDocument::METHOD => {
            let Ok(params) =
                serde_json::from_value::<DidChangeTextDocumentParams>(notification.params)
            else {
                return Ok(());
            };
            if !state.accepts_document_version(
                params.text_document.uri.as_str(),
                params.text_document.version,
            ) {
                return Ok(());
            }
            let uri = params.text_document.uri;
            let Some(text) = params
                .content_changes
                .into_iter()
                .last()
                .map(|change| change.text)
            else {
                return Ok(());
            };
            state.change_document(uri.as_str(), text);
            state.set_document_version(uri.as_str(), params.text_document.version);
            checking.schedule_document(
                uri.as_str(),
                Instant::now() + checking::EDIT_DEBOUNCE,
                true,
            );
            Ok(())
        }
        DidCloseTextDocument::METHOD => {
            let Ok(params) =
                serde_json::from_value::<DidCloseTextDocumentParams>(notification.params)
            else {
                return Ok(());
            };
            let uri = params.text_document.uri;
            state.close_document(uri.as_str());
            checking.forget_document(uri.as_str());
            publish_diagnostics(connection, uri, Vec::new(), None)?;
            checking.update_documents(state)
        }
        _ => Ok(()),
    }
}

fn show_configuration_error(connection: &Connection, message: String) -> Result<()> {
    connection.sender.send(Message::Notification(Notification {
        method: "window/showMessage".to_owned(),
        params: serde_json::json!({
            "type": 1,
            "message": format!("VersionLens configuration: {message}"),
        }),
    }))?;
    Ok(())
}

fn register_file_watcher(connection: &Connection, state: &VersionLensLspState) -> Result<()> {
    if !state.client.watched_files_dynamic {
        return Ok(());
    }
    connection
        .sender
        .send(Message::Request(Request {
            id: RequestId::from("versionlens/register-workspace-files".to_owned()),
            method: "client/registerCapability".to_owned(),
            params: serde_json::json!({
                "registrations": [{
                    "id": "versionlens-workspace-files",
                    "method": "workspace/didChangeWatchedFiles",
                    "registerOptions": {"watchers": [{"globPattern": "**/*"}]}
                }]
            }),
        }))
        .context("failed to register workspace file watching")
}

fn respond(connection: &Connection, id: RequestId, result: serde_json::Value) -> Result<()> {
    connection
        .sender
        .send(Message::Response(Response::new_ok(id, result)))
        .context("failed to send LSP response")
}

fn respond_error(
    connection: &Connection,
    id: RequestId,
    code: ErrorCode,
    message: String,
) -> Result<()> {
    connection
        .sender
        .send(Message::Response(Response::new_err(
            id,
            code as i32,
            message,
        )))
        .context("failed to send LSP error response")
}

fn publish_diagnostics(
    connection: &Connection,
    uri: Uri,
    diagnostics: Vec<Diagnostic>,
    version: Option<i32>,
) -> Result<()> {
    let mut params = VersionLensLspState::publish_diagnostics(uri, diagnostics);
    params.version = version;
    connection
        .sender
        .send(Message::Notification(Notification {
            method: "textDocument/publishDiagnostics".to_owned(),
            params: serde_json::to_value(params)?,
        }))
        .context("failed to publish LSP diagnostics")
}

#[cfg(test)]
mod tests;
