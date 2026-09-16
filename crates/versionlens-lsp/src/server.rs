use std::collections::HashMap;
use std::time::Duration;

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

use crate::state::{
    self, DISPLAY_CODE_LENS_COMMAND, UPDATE_DEPENDENCY_COMMAND, VersionLensLspState,
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
    initialization_options: Option<Value>,
    #[serde(default)]
    capabilities: lsp_types::ClientCapabilities,
    #[serde(flatten)]
    _remaining: HashMap<String, Value>,
}

struct DocumentChange {
    uri: Uri,
    diagnostics: Vec<Diagnostic>,
    version: i32,
}

mod checking;
mod transport;
mod work;
use checking::WorkspaceController;
use work::WorkQueue;

pub fn run_stdio_server() -> Result<()> {
    transport::run(|connection| run_connection(connection, true))
}

fn run_connection(connection: &Connection, persistent: bool) -> Result<()> {
    let mut state = initialize(connection, persistent)?;
    run_message_loop(connection, &mut state)
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
        let exclusions = state::workspace_exclusions(
            params
                .initialization_options
                .as_ref()
                .unwrap_or(&Value::Null),
        );
        let state = VersionLensLspState::with_workspace(
            params.root_uri,
            params.workspace_folders.unwrap_or_default(),
        );
        let state = state.with_client(params.capabilities);
        let state = match params.initialization_options {
            Some(value) => match state::session_configuration(
                value.get("versionlens").unwrap_or(&value).clone(),
            ) {
                Ok(config) => state.with_config(config),
                Err(error) => {
                    respond_error(
                        connection,
                        id,
                        ErrorCode::InvalidParams,
                        format!("invalid initialization options: {error}"),
                    )?;
                    continue;
                }
            },
            None => state,
        }
        .with_workspace_exclusions(exclusions);
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

fn run_message_loop(connection: &Connection, state: &mut VersionLensLspState) -> Result<()> {
    let mut work = WorkQueue::new()?;
    state.use_workspace_controller();
    let mut checking = WorkspaceController::start(state)?;
    let deadlines = crossbeam_channel::tick(Duration::from_millis(200));
    loop {
        crossbeam_channel::select! {
            recv(deadlines) -> now => {
                if let Ok(now) = now { work.expire_applications(connection, now)?; }
            }
            recv(work.receiver) -> completion => {
                if let Ok(completion) = completion { work.complete(state, connection, completion)?; }
            }
            recv(checking.receiver) -> event => {
                if let Ok(event) = event { checking.handle(state, connection, event)?; }
            }
            recv(connection.receiver) -> message => {
                let Ok(message) = message else { break };
                match message {
                    Message::Request(request) => {
                        if connection.handle_shutdown(&request)? { break; }
                        handle_request(connection, state, &mut work, request)?;
                    }
                    Message::Notification(notification) => {
                        if notification.method == "exit" { break; }
                        if notification.method == "$/cancelRequest" {
                            if let Some(id) = notification.params.get("id").and_then(|id| serde_json::from_value::<RequestId>(id.clone()).ok()) { work.cancel(connection, &id)?; }
                        } else {
                            handle_notification(connection, state, &mut checking, notification)?;
                            work.invalidate(state, connection)?;
                        }
                    }
                    Message::Response(response) => work.applied(connection, response)?,
                }
            }
        }
    }
    state.session.cancel_pending_resolutions();
    Ok(())
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
        "workspace/didChangeConfiguration" => {
            let Some(settings) = notification.params.get("settings") else {
                return Ok(());
            };
            change_configuration(connection, state, checking, settings)
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
            let diagnostics = state.open_document(VersionLensTextDocument {
                uri: uri.to_string(),
                language_id: params.text_document.language_id,
                text: params.text_document.text,
                workspace_root: None,
            });
            finish_document_change(
                connection,
                state,
                checking,
                DocumentChange {
                    uri,
                    diagnostics,
                    version: params.text_document.version,
                },
            )
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
            let diagnostics = state.change_document(uri.as_str(), text);
            finish_document_change(
                connection,
                state,
                checking,
                DocumentChange {
                    uri,
                    diagnostics,
                    version: params.text_document.version,
                },
            )
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
            checking.replace(state)
        }
        _ => Ok(()),
    }
}

fn change_configuration(
    connection: &Connection,
    state: &mut VersionLensLspState,
    checking: &mut WorkspaceController,
    settings: &Value,
) -> Result<()> {
    let exclusions = state::workspace_exclusions(settings);
    let session_settings = settings.get("versionlens").cloned().or_else(|| {
        settings.as_object().and_then(|settings| {
            settings
                .keys()
                .any(|key| {
                    matches!(
                        key.as_str(),
                        "cacheDurationMinutes"
                            | "cacheTtlSeconds"
                            | "cacheTtlMs"
                            | "enabledProviders"
                            | "providers"
                            | "suggestionIndicators"
                            | "showVulnerabilities"
                            | "showSuggestionStats"
                            | "showPrereleases"
                            | "http"
                    )
                })
                .then(|| Value::Object(settings.clone()))
        })
    });
    let config = match session_settings.map(state::session_configuration) {
        Some(Ok(config)) => Some(config),
        None => None,
        Some(Err(message)) => {
            show_configuration_error(connection, message)?;
            return Ok(());
        }
    };
    let session_changed = config
        .as_ref()
        .is_some_and(|config| !state.configuration_is(config));
    if let Some(config) = config
        && session_changed
        && let Err(error) = state.replace_configuration(config)
    {
        show_configuration_error(connection, error.to_string())?;
        return Ok(());
    }
    let exclusions_changed = state.replace_workspace_exclusions(exclusions);
    if session_changed {
        checking.restart(state).map_err(Into::into)
    } else if exclusions_changed {
        checking.replace(state)
    } else {
        Ok(())
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

fn finish_document_change(
    connection: &Connection,
    state: &mut VersionLensLspState,
    checking: &mut WorkspaceController,
    change: DocumentChange,
) -> Result<()> {
    let DocumentChange {
        uri,
        diagnostics,
        version,
    } = change;
    state.set_document_version(uri.as_str(), version);
    checking.observe_open_document(uri.as_str(), diagnostics.clone());
    publish_diagnostics(connection, uri, diagnostics, Some(version))?;
    checking.replace(state)
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
