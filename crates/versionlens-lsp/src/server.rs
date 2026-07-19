use std::collections::HashMap;

use anyhow::{Context, Result};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
};
use lsp_types::request::{CodeLensRequest, ExecuteCommand, Request as LspRequest};
use lsp_types::{
    CodeLensParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, ExecuteCommandParams, Uri, WorkspaceFolder,
};
use serde::Deserialize;
use serde_json::Value;

use crate::state::{
    DISPLAY_CODE_LENS_COMMAND, ResolvedDocument, VersionLensLspState, VersionLensTextDocument,
};

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeWorkspaceParams {
    #[serde(default)]
    root_uri: Option<Uri>,
    #[serde(default)]
    workspace_folders: Option<Vec<WorkspaceFolder>>,
    #[serde(flatten)]
    _remaining: HashMap<String, Value>,
}

pub fn run_stdio_server() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    run_connection(&connection)?;
    io_threads.join()?;
    Ok(())
}

fn run_connection(connection: &Connection) -> Result<()> {
    let mut state = initialize(connection)?;
    run_message_loop(connection, &mut state)
}

fn initialize(connection: &Connection) -> Result<VersionLensLspState> {
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
        );
        let initialize_result = serde_json::json!({
            "capabilities": VersionLensLspState::server_capabilities(),
        });
        connection.initialize_finish(id, initialize_result)?;
        return Ok(state);
    }
}

fn run_message_loop(connection: &Connection, state: &mut VersionLensLspState) -> Result<()> {
    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    break;
                }
                handle_request(connection, state, request)?;
            }
            Message::Notification(notification) => {
                if notification.method == "exit" {
                    break;
                }
                handle_notification(connection, state, notification)?;
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

fn handle_request(
    connection: &Connection,
    state: &VersionLensLspState,
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
            let resolved = state.resolve_document(params.text_document.uri.as_str());
            respond(
                connection,
                id,
                serde_json::to_value(
                    resolved
                        .as_ref()
                        .map_or(&[][..], |value| value.code_lenses.as_slice()),
                )?,
            )?;
            if let Some(ResolvedDocument { diagnostics, .. }) = resolved {
                publish_diagnostics(connection, params.text_document.uri, diagnostics)?;
            }
            Ok(())
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
            if params.command != DISPLAY_CODE_LENS_COMMAND || !params.arguments.is_empty() {
                return respond_error(
                    connection,
                    id,
                    ErrorCode::InvalidParams,
                    format!("unsupported command: {}", params.command),
                );
            }
            respond(connection, id, Value::Null)
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
            publish_diagnostics(connection, uri, diagnostics)
        }
        DidChangeTextDocument::METHOD => {
            let Ok(params) =
                serde_json::from_value::<DidChangeTextDocumentParams>(notification.params)
            else {
                return Ok(());
            };
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
            publish_diagnostics(connection, uri, diagnostics)
        }
        DidCloseTextDocument::METHOD => {
            let Ok(params) =
                serde_json::from_value::<DidCloseTextDocumentParams>(notification.params)
            else {
                return Ok(());
            };
            let uri = params.text_document.uri;
            state.close_document(uri.as_str());
            publish_diagnostics(connection, uri, Vec::new())
        }
        _ => Ok(()),
    }
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
    diagnostics: Vec<lsp_types::Diagnostic>,
) -> Result<()> {
    let params = VersionLensLspState::publish_diagnostics(uri, diagnostics);
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
