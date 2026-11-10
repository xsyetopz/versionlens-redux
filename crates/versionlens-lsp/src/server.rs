use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
};
use lsp_types::request::{
    CodeActionRequest, CodeLensRequest, ExecuteCommand, Request as LspRequest,
};
use lsp_types::{
    CodeLensParams, Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, ExecuteCommandParams, MessageType, Uri, WorkspaceFolder,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::state::{
    CLEAR_CACHE_COMMAND, DISPLAY_CODE_LENS_COMMAND, UPDATE_WORKSPACE_COMMAND, VersionLensLspState,
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

mod authorization;
mod checking;
mod rate_limit;
mod settings;
mod work;
use authorization::AuthorizationBroker;
use checking::{WorkspaceController, log_message};
use settings::SettingsRequests;
use work::WorkQueue;

/// Serves the language server over stdin and stdout until the client exits.
///
/// # Errors
///
/// Returns an error when the connection loop fails or an I/O thread cannot be joined.
pub fn run_stdio_server() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    run_connection(
        &connection,
        Some(versionlens_engine::application_cache_directory()),
    )?;
    drop(connection);
    io_threads.join()?;
    Ok(())
}

/// Runs the server on `connection`. `cache_directory` is where results are
/// stored across restarts; `None` keeps them in memory only.
fn run_connection(
    connection: &Connection,
    cache_directory: Option<std::io::Result<PathBuf>>,
) -> Result<()> {
    run_connection_with(connection, cache_directory, authorization::ANSWER_TIMEOUT)
}

/// [`run_connection`] with the time the client has to answer
/// `versionlens/authorization`.
fn run_connection_with(
    connection: &Connection,
    cache_directory: Option<std::io::Result<PathBuf>>,
    authorization_timeout: std::time::Duration,
) -> Result<()> {
    let mut state = initialize(connection, cache_directory)?;
    let mut settings = SettingsRequests::default();
    let queued = settings.first(connection, &mut state)?;
    run_message_loop(
        connection,
        &mut state,
        settings,
        queued,
        authorization_timeout,
    )
}

fn initialize(
    connection: &Connection,
    mut cache_directory: Option<std::io::Result<PathBuf>>,
) -> Result<VersionLensLspState> {
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
        let mut state = state;
        // An unwritable cache folder must not stop the server.
        let cache_error = cache_directory.take().and_then(|directory| {
            directory
                .and_then(|path| state.use_persistent_cache(&path))
                .err()
        });
        let initialize_result = serde_json::json!({
            "capabilities": VersionLensLspState::server_capabilities(),
        });
        connection.initialize_finish(id, initialize_result)?;
        if let Some(error) = cache_error {
            log_message(
                connection,
                MessageType::WARNING,
                format!(
                    "persistent cache is disabled, so results are kept in memory only: {error}"
                ),
            )?;
        }
        register_file_watcher(connection, &state)?;
        return Ok(state);
    }
}

/// Answers the open authorization questions when the loop ends, however it
/// ends.
struct CloseOnDrop(Arc<AuthorizationBroker>);

impl Drop for CloseOnDrop {
    fn drop(&mut self) {
        self.0.close();
    }
}

fn run_message_loop(
    connection: &Connection,
    state: &mut VersionLensLspState,
    mut settings: SettingsRequests,
    queued: Vec<Message>,
    authorization_timeout: std::time::Duration,
) -> Result<()> {
    let runtime = Runtime::new().context("failed to create LSP async runtime")?;
    let mut work = WorkQueue::new(runtime.handle().clone());
    state.use_workspace_controller();
    let authorization = AuthorizationBroker::new(
        connection.sender.clone(),
        runtime.handle().clone(),
        authorization_timeout,
    );
    let _closing = CloseOnDrop(Arc::clone(&authorization));
    state.use_authorization_source(Arc::<AuthorizationBroker>::clone(&authorization));
    let mut checking = WorkspaceController::start(state, runtime.handle().clone())?;
    let mut parts = LoopParts {
        state,
        work: &mut work,
        checking: &mut checking,
        settings: &mut settings,
        authorization: &authorization,
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
    parts.state.cancel_pending_resolutions();
    runtime.shutdown_background();
    Ok(())
}

/// The parts of the server that a client message can change.
struct LoopParts<'a> {
    state: &'a mut VersionLensLspState,
    work: &'a mut WorkQueue,
    checking: &'a mut WorkspaceController,
    settings: &'a mut SettingsRequests,
    authorization: &'a AuthorizationBroker,
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
        authorization,
    } = parts;
    match message {
        Message::Request(request) => {
            if connection.handle_shutdown(&request)? {
                return Ok(true);
            }
            if is_command(&request, CLEAR_CACHE_COMMAND) {
                clear_cache(connection, state, work, checking, request)?;
            } else if is_command(&request, UPDATE_WORKSPACE_COMMAND) && settings.pending() {
                settings.held.push(request);
            } else {
                handle_request(connection, state, work, request)?;
            }
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
            method => {
                // New folders need their settings.
                let folders_changed = method == "workspace/didChangeWorkspaceFolders";
                handle_notification(connection, state, checking, notification)?;
                work.invalidate(state, connection)?;
                if folders_changed {
                    settings.request(connection, state)?;
                }
            }
        },
        Message::Response(response) if settings::is_answer(&response.id) => {
            settings.answer(connection, state, Some(checking), response)?;
            work.invalidate(state, connection)?;
            if !settings.pending() {
                for request in std::mem::take(&mut settings.held) {
                    handle_request(connection, state, work, request)?;
                }
            }
        }
        Message::Response(response) if authorization::is_answer(&response.id) => {
            if let Some(late) = authorization.answer(response) {
                // Nothing was waiting, so the dependencies that were refused
                // are checked again with the credential.
                state.set_client_credential(&late.host, &late.credential);
                if let Err(error) = state.clear_caches() {
                    log_message(
                        connection,
                        MessageType::WARNING,
                        format!("VersionLens could not clear its cache: {error}"),
                    )?;
                }
                work.invalidate(state, connection)?;
                checking.restart(state, connection)?;
            }
        }
        Message::Response(response) => work.applied(connection, response)?,
    }
    Ok(false)
}

fn is_command(request: &Request, command: &str) -> bool {
    request.method == ExecuteCommand::METHOD
        && request.params.get("command").and_then(Value::as_str) == Some(command)
}

/// Clears the caches of every session and checks the workspace again, so the
/// next resolution fetches fresh registry data. It changes the server state,
/// so it runs here and not with the document commands.
fn clear_cache(
    connection: &Connection,
    state: &mut VersionLensLspState,
    work: &mut WorkQueue,
    checking: &mut WorkspaceController,
    request: Request,
) -> Result<()> {
    let arguments = request
        .params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Null);
    let no_arguments = match &arguments {
        Value::Null => true,
        Value::Array(arguments) => {
            arguments.is_empty()
                || matches!(arguments.as_slice(), [Value::Object(object)] if object.is_empty())
        }
        _ => false,
    };
    if !no_arguments {
        return respond_error(
            connection,
            request.id,
            ErrorCode::InvalidParams,
            format!("{CLEAR_CACHE_COMMAND} takes no arguments"),
        );
    }
    if let Err(error) = state.clear_caches() {
        return respond_error(
            connection,
            request.id,
            ErrorCode::RequestFailed,
            format!("failed to clear the cache: {error}"),
        );
    }
    work.invalidate(state, connection)?;
    if let Err(error) = checking.restart(state, connection) {
        return respond_error(
            connection,
            request.id,
            ErrorCode::RequestFailed,
            format!("cache cleared, but checking did not restart: {error}"),
        );
    }
    respond(connection, request.id, Value::Null)
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
        CodeActionRequest::METHOD => {
            let params = match serde_json::from_value::<lsp_types::CodeActionParams>(params) {
                Ok(params) => params,
                Err(error) => {
                    return respond_error(
                        connection,
                        id,
                        ErrorCode::InvalidParams,
                        format!("invalid {} params: {error}", CodeActionRequest::METHOD),
                    );
                }
            };
            work.actions(
                state,
                connection,
                id,
                params.text_document.uri.as_str(),
                params.range,
            )
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
                command => work.command(state, connection, id, command, params.arguments),
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
            state.change_workspace_folders(params.event)?;
            checking.restart(state, connection)
        }
        "workspace/didChangeWatchedFiles" => {
            let Ok(params) = serde_json::from_value::<lsp_types::DidChangeWatchedFilesParams>(
                notification.params,
            ) else {
                return Ok(());
            };
            // An ignore file can hide or reveal any manifest, so only its
            // change needs a full rescan.
            if params
                .changes
                .iter()
                .any(|change| state.watched_ignore_file(change.uri.as_str()))
            {
                state.invalidate_workspace();
                return checking.replace(state, connection);
            }
            checking.schedule_rescan(
                params
                    .changes
                    .iter()
                    .filter(|change| state.watched_file_is_relevant(change.uri.as_str()))
                    .filter_map(|change| versionlens_engine::workspace_path(change.uri.as_str())),
                Instant::now() + checking::WATCH_DEBOUNCE,
            );
            Ok(())
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
            checking.forget_document(connection, uri.as_str())?;
            publish_diagnostics(connection, uri, Vec::new(), None)?;
            checking.update_documents(state)
        }
        _ => Ok(()),
    }
}

fn show_configuration_error(connection: &Connection, message: &str) -> Result<()> {
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

fn respond(connection: &Connection, id: RequestId, result: Value) -> Result<()> {
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
mod tests {
    use std::thread;
    use std::time::Duration;

    use anyhow::{Context, Result, bail};
    use lsp_server::{Connection, ErrorCode, Message, Notification, Request};
    use lsp_types::{CodeLens, NumberOrString, PublishDiagnosticsParams, ServerCapabilities};
    use serde_json::{Value, json};

    use super::run_connection;
    use crate::state::DISPLAY_CODE_LENS_COMMAND;

    #[test]
    fn raw_loop_rejects_bad_requests_and_survives_bad_notifications() {
        let (server, client) = Connection::memory();
        let server_thread = thread::spawn(move || run_connection(&server, None));

        send_request(&client, 1, "initialize", Value::String("bad".to_owned()))
            .expect("test setup");
        assert_error(
            receive(&client).expect("test setup"),
            ErrorCode::InvalidParams,
        );

        initialize_client(&client);

        send_request(&client, 3, "textDocument/codeLens", json!({"bad": true}))
            .expect("test setup");
        assert_error(
            receive(&client).expect("test setup"),
            ErrorCode::InvalidParams,
        );

        send_request(&client, 4, "versionlens/unknown", json!({})).expect("test setup");
        assert_error(
            receive(&client).expect("test setup"),
            ErrorCode::MethodNotFound,
        );

        send_notification(&client, "textDocument/didOpen", Value::Bool(false)).expect("test setup");
        send_notification(&client, "textDocument/didChange", json!({"bad": true}))
            .expect("test setup");
        send_notification(&client, "versionlens/unknown", json!({"bad": true}))
            .expect("test setup");

        let uri = "file:///workspace/project/jsr.json";
        send_notification(
            &client,
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "json",
                    "version": 1,
                    "text": "{\"name\":\"@scope/pkg\",\"version\":\"1.0.0\"}"
                }
            }),
        )
        .expect("test setup");
        assert_diagnostics(receive(&client).expect("test setup"), uri, true);

        send_notification(
            &client,
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": uri, "version": 2},
                "contentChanges": []
            }),
        )
        .expect("test setup");
        send_notification(
            &client,
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": uri, "version": 3},
                "contentChanges": [{"text": "{\"name\":\"@scope/pkg\",\"version\":\"1.1.0\"}"}]
            }),
        )
        .expect("test setup");
        assert_diagnostics(receive(&client).expect("test setup"), uri, true);

        assert_visible_code_lenses(&client, uri);

        send_notification(
            &client,
            "textDocument/didClose",
            json!({"textDocument": {"uri": uri}}),
        )
        .expect("test setup");
        assert_diagnostics(receive(&client).expect("test setup"), uri, true);

        send_request(
            &client,
            8,
            "textDocument/codeLens",
            json!({"textDocument": {"uri": uri}}),
        )
        .expect("test setup");
        let lenses = serde_json::from_value::<Vec<CodeLens>>(
            ok_result(receive(&client).expect("test setup")).expect("test setup"),
        )
        .expect("test setup");
        assert_eq!(lenses.len(), 0);

        send_request(&client, 9, "shutdown", Value::Null).expect("test setup");
        let _shutdown = ok_result(receive(&client).expect("test setup")).expect("test setup");
        send_notification(&client, "exit", Value::Null).expect("test setup");

        let server_result = server_thread.join();
        assert!(server_result.is_ok());
        if let Ok(result) = server_result {
            result.expect("test setup");
        }
    }

    #[test]
    fn dynamically_registers_workspace_file_watching_when_supported() {
        let (server, client) = Connection::memory();
        let server_thread = thread::spawn(move || run_connection(&server, None));
        initialize_workspace(
            &client,
            json!({
                "capabilities": {
                    "workspace": {
                        "didChangeWatchedFiles": {"dynamicRegistration": true}
                    }
                }
            }),
        )
        .expect("test setup");
        let Message::Request(registration) = receive(&client).expect("test setup") else {
            panic!("expected dynamic file watcher registration");
        };
        assert_eq!(registration.method, "client/registerCapability");
        assert_eq!(
            registration.params["registrations"][0]["method"],
            "workspace/didChangeWatchedFiles"
        );
        assert_eq!(
            registration.params["registrations"][0]["registerOptions"]["watchers"][0]["globPattern"],
            "**/*"
        );
        client
            .sender
            .send(Message::Response(lsp_server::Response::new_ok(
                registration.id,
                Value::Null,
            )))
            .expect("test setup");
        shutdown_client(&client, 2, server_thread).expect("test setup");
    }

    #[test]
    fn publishes_unversioned_results_for_unopened_workspace_documents() {
        let root = versionlens_test_support::temporary_directory("versionlens-lsp-unopened")
            .expect("test setup");
        let workflow_directory = root.join(".github/workflows");
        std::fs::create_dir_all(&workflow_directory).expect("test setup");
        let workflow = workflow_directory.join("ci.yml");
        std::fs::write(&workflow, "jobs: {check: {steps: [{uses: './missing'}]}}\n")
            .expect("test setup");
        let root_uri = versionlens_engine::workspace_file_uri(&root)
            .context("missing root URI")
            .expect("test setup");
        let workflow_uri =
            versionlens_engine::workspace_file_uri(&workflow.canonicalize().expect("test setup"))
                .context("missing workflow URI")
                .expect("test setup");
        let (client, server_thread) = start_configured_server(
            json!({"rootUri": root_uri, "capabilities": {}}),
            json!({"suggestions": {"showVulnerabilities": false}, "workspace": {"check": "workspace"}}),
        ).expect("test setup");
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let message = client
                .receiver
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .expect("test setup");
            let Message::Notification(notification) = message else {
                continue;
            };
            if notification.method != "textDocument/publishDiagnostics" {
                continue;
            }
            let params = serde_json::from_value::<PublishDiagnosticsParams>(notification.params)
                .expect("test setup");
            if params.uri.as_str() == workflow_uri {
                assert_eq!(params.diagnostics.len(), 0);
                assert_eq!(params.version, None);
                break;
            }
        }
        shutdown_client(&client, 2, server_thread).expect("test setup");
        std::fs::remove_dir_all(root).expect("test setup");
    }

    #[test]
    fn discovery_failures_are_logged_once_and_never_published() {
        let root = versionlens_test_support::temporary_directory("versionlens-lsp-retry")
            .expect("test setup");
        std::fs::remove_dir_all(&root).expect("test setup");
        let root_uri = versionlens_engine::workspace_file_uri(&root)
            .context("missing root URI")
            .expect("test setup");
        let (client, server_thread) = start_configured_server(
            json!({"rootUri": root_uri, "capabilities": {}}),
            json!({"workspace": {"check": "workspace"}}),
        )
        .expect("test setup");

        // The missing root fails discovery, which retries after 1 s; the retry
        // fails the same way and must not be logged again.
        let mut logged = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_millis(2500);
        while let Some(wait) = deadline.checked_duration_since(std::time::Instant::now()) {
            let Ok(message) = client.receiver.recv_timeout(wait) else {
                break;
            };
            let Message::Notification(notification) = message else {
                panic!("unexpected message {message:?}");
            };
            match notification.method.as_str() {
                "window/logMessage" => logged.push(notification.params),
                "versionlens/status" => {}
                method => panic!("unexpected notification {method}"),
            }
        }
        assert_eq!(logged.len(), 1, "{logged:?}");
        assert_eq!(logged[0]["type"], 2);

        shutdown_client(&client, 2, server_thread).expect("test setup");
    }

    fn initialize_client(client: &Connection) {
        send_request(
            client,
            2,
            "initialize",
            json!({
                "processId": null,
                "rootUri": "file:///workspace",
                "workspaceFolders": [{
                    "uri": "file:///workspace/project",
                    "name": "project"
                }],
                "capabilities": {"workspace":{"applyEdit":true,"workspaceEdit":{"documentChanges":true}}}
            }),
        ).expect("test setup");
        let initialize_result =
            ok_result(receive(client).expect("test setup")).expect("test setup");
        let capabilities =
            serde_json::from_value::<ServerCapabilities>(initialize_result["capabilities"].clone())
                .expect("test setup");
        assert!(capabilities.code_lens_provider.is_some());
        assert!(capabilities.code_action_provider.is_some());
        let execute_commands = capabilities
            .execute_command_provider
            .ok_or_else(|| anyhow::anyhow!("expected execute command capabilities"))
            .expect("test setup");
        assert_eq!(
            execute_commands.commands,
            [
                "versionlens.displayCodeLens",
                "versionlens.server.update",
                "versionlens.server.updateAll",
                "versionlens.server.updateWorkspace",
                "versionlens.server.sort",
                "versionlens.server.chooseBuild",
                "versionlens.server.clearCache",
                "versionlens.server.manifestInfo",
            ]
        );
        send_notification(client, "initialized", json!({})).expect("test setup");
    }

    fn initialize_workspace(client: &Connection, params: Value) -> Result<Value> {
        send_request(client, 1, "initialize", params)?;
        let result = ok_result(receive(client)?)?;
        send_notification(client, "initialized", json!({}))?;
        Ok(result)
    }

    /// Initializes a client that answers `workspace/configuration` with
    /// `settings` for the `versionlens` section and `null` for the others.
    fn initialize_configured(
        client: &Connection,
        mut params: Value,
        settings: Value,
    ) -> Result<Value> {
        params["capabilities"]["workspace"]["configuration"] = json!(true);
        let result = initialize_workspace(client, params)?;
        answer_configuration(client, settings)?;
        Ok(result)
    }

    /// Answers the next `workspace/configuration` request and returns its
    /// params.
    fn answer_configuration(client: &Connection, settings: Value) -> Result<Value> {
        loop {
            match client.receiver.recv_timeout(Duration::from_secs(10))? {
                Message::Request(request) if request.method == "workspace/configuration" => {
                    let items = request.params["items"].as_array().map_or(0, Vec::len);
                    let mut result = vec![Value::Null; items];
                    if let Some(first) = result.first_mut() {
                        *first = settings;
                    }
                    client
                        .sender
                        .send(Message::Response(lsp_server::Response::new_ok(
                            request.id, result,
                        )))?;
                    return Ok(request.params);
                }
                Message::Notification(notification)
                    if matches!(
                        notification.method.as_str(),
                        "window/logMessage" | "versionlens/status"
                    ) => {}
                message => bail!("expected a configuration request, got {message:?}"),
            }
        }
    }

    fn start_configured_server(
        params: Value,
        settings: Value,
    ) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
        let (server, client) = Connection::memory();
        let server_thread = thread::spawn(move || run_connection(&server, None));
        initialize_configured(&client, params, settings)?;
        Ok((client, server_thread))
    }

    fn start_workspace_server(
        params: Value,
    ) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
        let (server, client) = Connection::memory();
        let server_thread = thread::spawn(move || run_connection(&server, None));
        initialize_workspace(&client, params)?;
        Ok((client, server_thread))
    }

    fn assert_visible_code_lenses(client: &Connection, uri: &str) {
        send_request(
            client,
            5,
            "textDocument/codeLens",
            json!({"textDocument": {"uri": uri}}),
        )
        .expect("test setup");
        let lenses = serde_json::from_value::<Vec<CodeLens>>(
            ok_result(receive(client).expect("test setup")).expect("test setup"),
        )
        .expect("test setup");
        crate::test_support::assert_update_code_lenses(&lenses);
        assert_diagnostics(receive(client).expect("test setup"), uri, true);

        send_request(
            client,
            6,
            "workspace/executeCommand",
            json!({"command": DISPLAY_CODE_LENS_COMMAND}),
        )
        .expect("test setup");
        assert_eq!(
            ok_result(receive(client).expect("test setup")).expect("test setup"),
            Value::Null
        );

        send_request(
            client,
            7,
            "workspace/executeCommand",
            json!({"command": "versionlens.update", "arguments": []}),
        )
        .expect("test setup");
        assert_error(
            receive(client).expect("test setup"),
            ErrorCode::InvalidParams,
        );
    }

    fn send_request(connection: &Connection, id: i32, method: &str, params: Value) -> Result<()> {
        connection.sender.send(Message::Request(Request {
            id: id.into(),
            method: method.to_owned(),
            params,
        }))?;
        Ok(())
    }

    fn open_json_document(client: &Connection, uri: &str, version: i32, text: &str) -> Result<()> {
        send_notification(
            client,
            "textDocument/didOpen",
            json!({"textDocument": {
                "uri": uri, "languageId": "json", "version": version, "text": text,
            }}),
        )
    }

    fn checked_command(
        client: &Connection,
        id: i32,
        uri: &str,
        version: &str,
    ) -> Result<lsp_types::Command> {
        send_request(
            client,
            id,
            "textDocument/codeLens",
            json!({"textDocument":{"uri":uri}}),
        )?;
        let lenses = serde_json::from_value::<Vec<CodeLens>>(ok_result(receive(client)?)?)?;
        assert_diagnostics(receive(client)?, uri, true);
        lenses
            .into_iter()
            .filter_map(|lens| lens.command)
            .find(|command| command.title.contains(version))
            .ok_or_else(|| anyhow::anyhow!("missing checked version {version}"))
    }

    fn shutdown_client(
        client: &Connection,
        id: i32,
        server: thread::JoinHandle<Result<()>>,
    ) -> Result<()> {
        send_request(client, id, "shutdown", Value::Null)?;
        ok_result(receive(client)?)?;
        send_notification(client, "exit", Value::Null)?;
        server
            .join()
            .map_err(|_| anyhow::anyhow!("server panicked"))?
    }

    fn send_notification(connection: &Connection, method: &str, params: Value) -> Result<()> {
        connection.sender.send(Message::Notification(Notification {
            method: method.to_owned(),
            params,
        }))?;
        Ok(())
    }

    fn receive(connection: &Connection) -> Result<Message> {
        loop {
            let message = connection.receiver.recv_timeout(Duration::from_secs(10))?;
            if matches!(
                &message,
                Message::Notification(notification)
                    if matches!(notification.method.as_str(), "window/logMessage" | "versionlens/status")
            ) {
                continue;
            }
            return Ok(message);
        }
    }

    fn ok_result(message: Message) -> Result<Value> {
        let Message::Response(response) = message else {
            bail!("expected response, got {message:?}");
        };
        response
            .response_result
            .map_err(|error| anyhow::anyhow!("expected successful response, got {error:?}"))
    }

    fn assert_error(message: Message, expected_code: ErrorCode) {
        let Message::Response(response) = message else {
            panic!("expected response, got {message:?}");
        };
        let error = response
            .response_result
            .expect_err("expected error response");
        assert_eq!(error.code, expected_code as i32);
    }

    /// Asserts a diagnostics notification for `expected_uri`, and whether it has
    /// no diagnostic other than an available update.
    fn assert_diagnostics(message: Message, expected_uri: &str, empty: bool) {
        let Message::Notification(notification) = message else {
            panic!("expected diagnostics notification, got {message:?}");
        };
        assert_eq!(notification.method, "textDocument/publishDiagnostics");
        let params = serde_json::from_value::<PublishDiagnosticsParams>(notification.params)
            .expect("test setup");
        assert_eq!(params.uri.as_str(), expected_uri);
        let update = Some(NumberOrString::String(
            versionlens_engine::UPDATE_AVAILABLE.to_owned(),
        ));
        let noise = params
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code != update)
            .count();
        assert_eq!(noise == 0, empty, "{:?}", params.diagnostics);
    }

    mod updates {
        use super::*;

        #[test]
        fn project_version_command_sends_a_versioned_edit_and_waits_for_the_editor() {
            let (server, client) = Connection::memory();
            let server_thread = thread::spawn(move || run_connection(&server, None));
            initialize_client(&client);
            let uri = "file:///workspace/jsr.json";
            open_json_document(
                &client,
                uri,
                7,
                "{\"name\":\"@scope/pkg\",\"version\":\"1.0.0\"}",
            )
            .expect("test setup");
            assert_diagnostics(receive(&client).expect("test setup"), uri, true);
            let command = checked_command(&client, 20, uri, "1.0.1").expect("test setup");
            send_request(
                &client,
                21,
                "workspace/executeCommand",
                serde_json::to_value(&command).expect("test setup"),
            )
            .expect("test setup");
            let request = loop {
                let message = receive(&client).expect("test setup");
                match message {
                    Message::Request(request) => break request,
                    Message::Notification(notification)
                        if notification.method == "textDocument/publishDiagnostics" => {}
                    message => panic!("expected workspace edit request, got {message:?}"),
                }
            };
            assert_eq!(request.method, "workspace/applyEdit");
            let changes = &request.params["edit"]["documentChanges"];
            assert_eq!(changes[0]["textDocument"], json!({"uri":uri,"version":7}));
            assert_eq!(changes[0]["edits"][0]["newText"], "1.0.1");
            assert!(client.receiver.try_recv().is_err());
            client
                .sender
                .send(Message::Response(lsp_server::Response::new_ok(
                    request.id,
                    json!({"applied":true}),
                )))
                .expect("test setup");
            assert_eq!(
                ok_result(receive(&client).expect("test setup")).expect("test setup"),
                Value::Null
            );

            send_notification(
                &client,
                "textDocument/didChange",
                json!({"textDocument":{"uri":uri,"version":8},"contentChanges":[{"text":"{\"name\":\"@scope/pkg\",\"version\":\"2.0.0\"}"}]}),
            ).expect("test setup");
            assert_diagnostics(receive(&client).expect("test setup"), uri, true);
            // The lens was made for version 7. Clicked after the edit, its patch bump
            // is computed again from the current text, 2.0.0.
            send_request(
                &client,
                22,
                "workspace/executeCommand",
                serde_json::to_value(&command).expect("test setup"),
            )
            .expect("test setup");
            let Message::Request(request) = receive(&client).expect("test setup") else {
                panic!("expected a workspace edit");
            };
            let changes = &request.params["edit"]["documentChanges"];
            assert_eq!(changes[0]["textDocument"], json!({"uri":uri,"version":8}));
            assert_eq!(changes[0]["edits"][0]["newText"], "2.0.1");
            client
                .sender
                .send(Message::Response(lsp_server::Response::new_ok(
                    request.id,
                    json!({"applied":true}),
                )))
                .expect("test setup");
            assert_eq!(
                ok_result(receive(&client).expect("test setup")).expect("test setup"),
                Value::Null
            );
            shutdown_client(&client, 23, server_thread).expect("test setup");
        }

        #[test]
        fn shutdown_and_document_changes_do_not_wait_for_registry_requests() -> Result<()> {
            responsive_registry_request(false)
        }

        #[test]
        fn requests_can_be_cancelled_while_the_registry_is_pending() -> Result<()> {
            responsive_registry_request(true)
        }

        fn responsive_registry_request(cancel: bool) -> Result<()> {
            use std::io::{Read, Write};
            let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
            let address = listener.local_addr()?;
            let (started, request_started) = std::sync::mpsc::channel();
            let (release, wait_release) = std::sync::mpsc::channel();
            let registry = thread::spawn(move || -> Result<()> {
                let Some(mut socket) = versionlens_test_support::accept_within_deadline(&listener)
                else {
                    anyhow::bail!("registry request never arrived");
                };
                let mut buffer = [0; 4096];
                let received = socket.read(&mut buffer)?;
                anyhow::ensure!(received > 0, "registry request was empty");
                started.send(())?;
                wait_release.recv_timeout(Duration::from_secs(10))?;
                let body = r#"{"dist-tags":{"latest":"2.0.0"}}"#;
                drop(write!(
                    socket,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                ));
                Ok(())
            });
            let (server, client) = Connection::memory();
            let server_thread = thread::spawn(move || run_connection(&server, None));
            initialize_configured(
                &client,
                json!({"capabilities":{}}),
                json!({"suggestions":{"showVulnerabilities":false},"npm":{"apiUrl":format!("http://{address}")}}),
            )?;
            let uri = "file:///workspace/package.json";
            open_json_document(
                &client,
                uri,
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )?;
            send_request(
                &client,
                2,
                "textDocument/codeLens",
                json!({"textDocument":{"uri":uri}}),
            )?;
            request_started
                .recv_timeout(Duration::from_secs(3))
                .map_err(|error| {
                    anyhow::anyhow!(
                        "registry request did not start: {error}; server response: {:?}",
                        client.receiver.try_recv()
                    )
                })?;
            if cancel {
                send_notification(&client, "$/cancelRequest", json!({"id":2}))?;
                assert_error(next_response(&client)?, ErrorCode::RequestCanceled);
            } else {
                send_notification(
                    &client,
                    "textDocument/didChange",
                    json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":"{}"}]}),
                )?;
                assert_error(next_response(&client)?, ErrorCode::ContentModified);
            }
            shutdown_client(&client, 3, server_thread)?;
            release.send(())?;
            registry
                .join()
                .expect("registry panicked")
                .expect("test setup");
            Ok(())
        }

        /// The next response within one second. Notifications sent meanwhile, such as
        /// status and diagnostics from push resolution, are skipped.
        fn next_response(client: &Connection) -> Result<Message> {
            let deadline = std::time::Instant::now() + Duration::from_secs(1);
            loop {
                let message = client
                    .receiver
                    .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))?;
                if matches!(message, Message::Response(_)) {
                    return Ok(message);
                }
            }
        }
    }

    mod refresh {
        use std::io::{Read, Write};

        use super::*;

        #[test]
        fn opening_a_manifest_checks_versions_and_requests_a_refresh() {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("test setup");
            let address = listener.local_addr().expect("test setup");
            let registry = thread::spawn(move || -> Result<()> {
                let Some(mut socket) = versionlens_test_support::accept_within_deadline(&listener)
                else {
                    anyhow::bail!("registry request never arrived");
                };
                socket
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .expect("test setup");
                let mut buffer = [0; 4096];
                anyhow::ensure!(
                    socket.read(&mut buffer).expect("test setup") > 0,
                    "empty registry request"
                );
                let body = r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#;
                write!(
                    socket,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .expect("test setup");
                Ok(())
            });
            let (server, client) = Connection::memory();
            let server_thread = thread::spawn(move || run_connection(&server, None));
            initialize_configured(
                &client,
                json!({"capabilities":{"workspace":{"codeLens":{"refreshSupport":true},"applyEdit":true,"workspaceEdit":{"documentChanges":true}}}}),
                json!({"suggestions":{"showVulnerabilities":false},"npm":{"apiUrl":format!("http://{address}")}}),
            ).expect("test setup");
            let uri = "file:///workspace/package.json";
            open_json_document(
                &client,
                uri,
                3,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )
            .expect("test setup");
            assert_diagnostics(receive(&client).expect("test setup"), uri, true);
            let Message::Request(refresh) = receive(&client).expect("test setup") else {
                panic!("expected code lens refresh");
            };
            assert_eq!(refresh.method, "workspace/codeLens/refresh");
            client
                .sender
                .send(Message::Response(lsp_server::Response::new_ok(
                    refresh.id,
                    Value::Null,
                )))
                .expect("test setup");
            registry
                .join()
                .expect("registry panicked")
                .expect("test setup");

            let command = checked_command(&client, 2, uri, "2.0.0").expect("test setup");
            send_request(
                &client,
                3,
                "workspace/executeCommand",
                serde_json::to_value(command).expect("test setup"),
            )
            .expect("test setup");
            let Message::Request(edit) = receive(&client).expect("test setup") else {
                panic!("expected update edit");
            };
            assert_eq!(
                edit.params["edit"]["documentChanges"][0]["edits"][0]["newText"],
                "2.0.0"
            );
            client
                .sender
                .send(Message::Response(lsp_server::Response::new_ok(
                    edit.id,
                    json!({"applied":false,"failureReason":"document changed in editor"}),
                )))
                .expect("test setup");
            assert_error(
                receive(&client).expect("test setup"),
                ErrorCode::RequestFailed,
            );
            shutdown_client(&client, 4, server_thread).expect("test setup");
        }
    }

    mod responsiveness {
        use std::io::{Read, Write};
        use std::sync::mpsc;
        use std::time::Instant;

        use lsp_server::{RequestId, Response};

        use super::*;

        const RESPONSE_BUDGET: Duration = Duration::from_millis(500);

        #[test]
        fn other_documents_and_shutdown_do_not_wait_for_a_delayed_registry() -> Result<()> {
            let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
            let address = listener.local_addr()?;
            let (started, request_started) = mpsc::channel();
            // Detached on purpose: the delayed response is not part of what the test asserts.
            let _registry = thread::spawn(move || -> Result<()> {
                let Some(mut socket) = versionlens_test_support::accept_within_deadline(&listener)
                else {
                    anyhow::bail!("registry request never arrived");
                };
                let mut buffer = [0; 4096];
                anyhow::ensure!(socket.read(&mut buffer)? > 0, "registry request was empty");
                started.send(())?;
                thread::sleep(Duration::from_secs(5));
                let body = r#"{"dist-tags":{"latest":"2.0.0"}}"#;
                drop(write!(
                    socket,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                ));
                Ok(())
            });
            let (server, client) = Connection::memory();
            let server_thread = thread::spawn(move || run_connection(&server, None));
            initialize_configured(
                &client,
                json!({"capabilities":{}}),
                json!({"suggestions":{"showVulnerabilities":false},"npm":{"apiUrl":format!("http://{address}")}}),
            )?;

            let slow = "file:///workspace/slow/package.json";
            let other = "file:///workspace/other/package.json";
            open_json_document(
                &client,
                slow,
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )?;
            open_json_document(&client, other, 1, "{\"name\":\"other\"}")?;
            request_started
                .recv_timeout(Duration::from_secs(3))
                .map_err(|error| anyhow::anyhow!("registry request did not start: {error}"))?;

            send_request(
                &client,
                2,
                "textDocument/codeLens",
                json!({"textDocument":{"uri":other}}),
            )?;
            let lenses = await_response(&client, &RequestId::from(2), RESPONSE_BUDGET)?;
            anyhow::ensure!(lenses.is_array(), "expected code lens array, got {lenses}");

            send_request(&client, 3, "shutdown", Value::Null)?;
            await_response(&client, &RequestId::from(3), RESPONSE_BUDGET)?;
            send_notification(&client, "exit", Value::Null)?;
            server_thread
                .join()
                .map_err(|_| anyhow::anyhow!("server panicked"))?
        }

        fn await_response(client: &Connection, id: &RequestId, budget: Duration) -> Result<Value> {
            let sent_at = Instant::now();
            loop {
                let remaining = budget.saturating_sub(sent_at.elapsed());
                let message = client
                    .receiver
                    .recv_timeout(remaining)
                    .with_context(|| format!("no response to {id} within {budget:?}"))?;
                let Message::Response(Response {
                    id: response_id,
                    response_result,
                }) = message
                else {
                    continue;
                };
                if response_id != *id {
                    continue;
                }
                return response_result
                    .map_err(|error| anyhow::anyhow!("expected success for {id}, got {error:?}"));
            }
        }
    }

    mod incremental {
        use std::collections::BTreeMap;
        use std::io::{Read, Write};
        use std::sync::{Arc, Mutex};

        use super::*;

        /// Background checking reruns after each edit, so the test waits this long
        /// before it counts registry requests.
        const SETTLE: Duration = Duration::from_millis(1500);

        pub(super) type Requests = Arc<Mutex<BTreeMap<String, usize>>>;

        #[test]
        fn editing_a_manifest_resolves_only_changed_dependencies() {
            let (address, requests) = counting_registry().expect("test setup");
            let (server, client) = Connection::memory();
            let server_thread = thread::spawn(move || run_connection(&server, None));
            initialize_configured(
                &client,
                json!({"capabilities":{}}),
                json!({"suggestions":{"showVulnerabilities":false},"npm":{"apiUrl":format!("http://{address}")}}),
            ).expect("test setup");
            let uri = "file:///workspace/package.json";
            open_json_document(&client, uri, 1, &manifest("// first", "1.0.0"))
                .expect("test setup");
            lens_titles(&client, 10, uri).expect("test setup");
            thread::sleep(SETTLE);
            let resolved = snapshot(&requests);
            assert!(
                resolved.contains_key("/alpha") && resolved.contains_key("/beta"),
                "expected both packages to be fetched, got {resolved:?}"
            );

            change_document(&client, uri, 2, &manifest("// second", "1.0.0")).expect("test setup");
            let titles = lens_titles(&client, 11, uri).expect("test setup");
            thread::sleep(SETTLE);
            assert_eq!(
                snapshot(&requests),
                resolved,
                "a comment edit made registry requests"
            );
            assert!(
                titles.iter().any(|title| title.contains("2.0.0")),
                "suggestions disappeared after a comment edit: {titles:?}"
            );

            change_document(&client, uri, 3, &manifest("// second", "2.0.0")).expect("test setup");
            lens_titles(&client, 12, uri).expect("test setup");
            thread::sleep(SETTLE);
            let after = snapshot(&requests);
            assert_eq!(
                after.get("/beta"),
                resolved.get("/beta"),
                "an alpha requirement edit fetched beta again"
            );
            send_request(&client, 13, "shutdown", Value::Null).expect("test setup");
            loop {
                if let Message::Response(response) = receive(&client).expect("test setup") {
                    assert!(response.id == 13.into(), "unexpected response {response:?}");
                    break;
                }
            }
            send_notification(&client, "exit", Value::Null).expect("test setup");
            server_thread
                .join()
                .expect("server panicked")
                .expect("test setup");
        }

        fn manifest(comment: &str, alpha: &str) -> String {
            format!(
                "{{\n  {comment}\n  \"dependencies\": {{\"alpha\": \"{alpha}\", \"beta\": \"1.0.0\"}}\n}}"
            )
        }

        fn change_document(client: &Connection, uri: &str, version: i32, text: &str) -> Result<()> {
            send_notification(
                client,
                "textDocument/didChange",
                json!({
                    "textDocument": {"uri": uri, "version": version},
                    "contentChanges": [{"text": text}],
                }),
            )
        }

        pub(super) fn lens_titles(client: &Connection, id: i32, uri: &str) -> Result<Vec<String>> {
            send_request(
                client,
                id,
                "textDocument/codeLens",
                json!({"textDocument":{"uri":uri}}),
            )?;
            loop {
                let message = receive(client)?;
                let Message::Response(response) = message else {
                    continue;
                };
                if response.id != id.into() {
                    continue;
                }
                let lenses =
                    serde_json::from_value::<Vec<CodeLens>>(response.response_result.map_err(
                        |error| anyhow::anyhow!("code lens request failed: {error:?}"),
                    )?)?;
                return Ok(lenses
                    .into_iter()
                    .filter_map(|lens| lens.command.map(|command| command.title))
                    .collect());
            }
        }

        fn snapshot(requests: &Requests) -> BTreeMap<String, usize> {
            requests
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        }

        /// Serves every package with `latest` 2.0.0 and counts requests by path.
        fn counting_registry() -> Result<(std::net::SocketAddr, Requests)> {
            serving_registry(
                r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#,
            )
        }

        /// Serves `body` for every package and counts requests by path.
        pub(super) fn serving_registry(
            body: &'static str,
        ) -> Result<(std::net::SocketAddr, Requests)> {
            let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
            let address = listener.local_addr()?;
            let requests = Requests::default();
            let counter = Arc::clone(&requests);
            // Detached on purpose: it serves until the test process ends.
            thread::spawn(move || {
                for socket in listener.incoming() {
                    let Ok(mut socket) = socket else {
                        continue;
                    };
                    drop(socket.set_read_timeout(Some(Duration::from_secs(3))));
                    let mut buffer = [0; 4096];
                    let Ok(read) = socket.read(&mut buffer) else {
                        continue;
                    };
                    let request = String::from_utf8_lossy(&buffer[..read]);
                    let path = request
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or_default()
                        .to_owned();
                    *counter
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .entry(path)
                        .or_default() += 1;
                    drop(write!(
                        socket,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    ));
                }
            });
            Ok((address, requests))
        }
    }

    mod push {
        use std::io::{Read, Write};
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

        use super::*;

        /// Serves npm metadata for every request on one socket per request, and
        /// marks `requested` before it answers, so a test can tell whether a message
        /// was sent before or after resolution read the registry.
        fn npm_registry(
            requested: Arc<AtomicBool>,
            count: Arc<AtomicUsize>,
        ) -> Result<std::net::SocketAddr> {
            let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
            let address = listener.local_addr()?;
            thread::spawn(move || {
                for socket in listener.incoming() {
                    let Ok(mut socket) = socket else { return };
                    drop(socket.set_read_timeout(Some(Duration::from_secs(3))));
                    let mut buffer = [0; 4096];
                    if socket.read(&mut buffer).unwrap_or(0) == 0 {
                        continue;
                    }
                    count.fetch_add(1, Ordering::SeqCst);
                    requested.store(true, Ordering::SeqCst);
                    let body =
                        r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#;
                    drop(write!(
                        socket,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    ));
                }
            });
            Ok(address)
        }

        fn start_without_code_lens(
            address: std::net::SocketAddr,
        ) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
            start_configured_server(
                json!({"capabilities": {}}),
                json!({"suggestions": {"showVulnerabilities": false}, "npm": {"apiUrl": format!("http://{address}")}}),
            )
        }

        enum Seen {
            Status(String),
            Diagnostics {
                version: Option<i32>,
                after_registry: bool,
            },
        }

        /// Collects the status and diagnostics messages for `uri` until an idle
        /// status arrives after at least one busy status.
        fn collect_until_idle(
            client: &Connection,
            uri: &str,
            requested: &AtomicBool,
        ) -> Result<Vec<Seen>> {
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            let mut seen = Vec::new();
            loop {
                let message = client
                    .receiver
                    .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))?;
                let Some(next) = seen_message(message, uri, requested) else {
                    continue;
                };
                let idle = matches!(&next, Seen::Status(status) if status == "idle");
                seen.push(next);
                if idle
                    && seen
                        .iter()
                        .any(|seen| matches!(seen, Seen::Status(status) if status == "busy"))
                {
                    return Ok(seen);
                }
            }
        }

        /// Collects the status and diagnostics messages for `uri` that arrive
        /// within `window`.
        fn collect_within(
            client: &Connection,
            uri: &str,
            requested: &AtomicBool,
            window: Duration,
        ) -> Vec<Seen> {
            let deadline = std::time::Instant::now() + window;
            let mut seen = Vec::new();
            while let Ok(message) = client
                .receiver
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
            {
                seen.extend(seen_message(message, uri, requested));
            }
            seen
        }

        fn seen_message(message: Message, uri: &str, requested: &AtomicBool) -> Option<Seen> {
            let Message::Notification(notification) = message else {
                return None;
            };
            if notification.params["uri"] != uri {
                return None;
            }
            match notification.method.as_str() {
                "versionlens/status" => Some(Seen::Status(
                    notification.params["status"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                )),
                "textDocument/publishDiagnostics" => Some(Seen::Diagnostics {
                    version: notification.params["version"]
                        .as_i64()
                        .and_then(|version| i32::try_from(version).ok()),
                    after_registry: requested.load(Ordering::SeqCst),
                }),
                _ => None,
            }
        }

        fn statuses(seen: &[Seen]) -> Vec<&str> {
            seen.iter()
                .filter_map(|seen| match seen {
                    Seen::Status(status) => Some(status.as_str()),
                    Seen::Diagnostics { .. } => None,
                })
                .collect()
        }

        #[test]
        fn a_client_without_code_lens_support_gets_diagnostics_once_resolution_completes() {
            let requested = Arc::new(AtomicBool::new(false));
            let address = npm_registry(Arc::clone(&requested), Arc::new(AtomicUsize::new(0)))
                .expect("test setup");
            let (client, server_thread) = start_without_code_lens(address).expect("test setup");
            let uri = "file:///workspace/push/package.json";

            open_json_document(
                &client,
                uri,
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )
            .expect("test setup");
            let seen = collect_until_idle(&client, uri, &requested).expect("test setup");

            assert_eq!(statuses(&seen), ["busy", "idle"]);
            let diagnostics = seen
                .iter()
                .filter_map(|seen| match seen {
                    Seen::Diagnostics {
                        version,
                        after_registry,
                    } => Some((*version, *after_registry)),
                    Seen::Status(_) => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(diagnostics, [(Some(1), true)]);
            shutdown_client(&client, 2, server_thread).expect("test setup");
        }

        #[test]
        fn rapid_edits_are_resolved_once_after_the_debounce() {
            let requested = Arc::new(AtomicBool::new(false));
            let address = npm_registry(Arc::clone(&requested), Arc::new(AtomicUsize::new(0)))
                .expect("test setup");
            let (client, server_thread) = start_without_code_lens(address).expect("test setup");
            let uri = "file:///workspace/debounce/package.json";
            open_json_document(
                &client,
                uri,
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )
            .expect("test setup");
            collect_until_idle(&client, uri, &requested).expect("test setup");

            for (version, requirement) in [(2, "^1.0.0"), (3, "~1.0.0")] {
                send_notification(
                    &client,
                    "textDocument/didChange",
                    json!({
                        "textDocument": {"uri": uri, "version": version},
                        "contentChanges": [{"text": format!("{{\"dependencies\":{{\"example\":\"{requirement}\"}}}}")}]
                    }),
                ).expect("test setup");
            }
            let early = collect_within(&client, uri, &requested, Duration::from_millis(200));
            assert!(
                early.is_empty(),
                "the edits were checked before the debounce ended: {} messages",
                early.len()
            );
            let seen = collect_until_idle(&client, uri, &requested).expect("test setup");

            assert_eq!(statuses(&seen), ["busy", "idle"]);
            let versions = seen
                .iter()
                .filter_map(|seen| match seen {
                    Seen::Diagnostics { version, .. } => Some(*version),
                    Seen::Status(_) => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(versions, [Some(3)]);
            shutdown_client(&client, 2, server_thread).expect("test setup");
        }

        #[test]
        fn an_edit_checked_early_by_a_workspace_reload_does_not_stay_busy() {
            let requested = Arc::new(AtomicBool::new(false));
            let address = npm_registry(Arc::clone(&requested), Arc::new(AtomicUsize::new(0)))
                .expect("test setup");
            let (client, server_thread) = start_without_code_lens(address).expect("test setup");
            let uri = "file:///workspace/reload/package.json";
            open_json_document(
                &client,
                uri,
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )
            .expect("test setup");
            collect_until_idle(&client, uri, &requested).expect("test setup");

            send_notification(
                &client,
                "textDocument/didChange",
                json!({
                    "textDocument": {"uri": uri, "version": 2},
                    "contentChanges": [{"text": "{\"dependencies\":{\"example\":\"^1.0.0\"}}"}]
                }),
            )
            .expect("test setup");
            // A reload sends the unsaved text to checking before the debounce ends.
            send_notification(
                &client,
                "workspace/didChangeWorkspaceFolders",
                json!({"event": {"added": [], "removed": []}}),
            )
            .expect("test setup");
            let seen = collect_within(&client, uri, &requested, Duration::from_secs(2));

            assert_ne!(
                statuses(&seen).last(),
                Some(&"busy"),
                "the document stayed busy"
            );
            shutdown_client(&client, 2, server_thread).expect("test setup");
        }
    }

    mod configuration {
        use super::*;

        #[test]
        fn settings_are_pulled_for_the_folder_and_pulled_again_on_change() {
            let (server, client) = Connection::memory();
            let server_thread = thread::spawn(move || run_connection(&server, None));
            let folder = "file:///workspace/project";
            initialize_workspace(
                &client,
                json!({
                    "workspaceFolders": [{"uri": folder, "name": "project"}],
                    "capabilities": {"workspace": {"configuration": true}}
                }),
            )
            .expect("test setup");
            let params = answer_configuration(&client, json!({})).expect("test setup");
            assert_eq!(
                params,
                json!({"items": [
                    {"scopeUri": folder, "section": "versionlens"},
                    {"scopeUri": folder, "section": "files.exclude"}
                ]})
            );

            // The notification payload is ignored; the server asks again.
            send_notification(
                &client,
                "workspace/didChangeConfiguration",
                json!({"settings": {"versionlens": {"caching": {"duration": "invalid"}}}}),
            )
            .expect("test setup");
            let params = answer_configuration(&client, json!({"caching": {"duration": "invalid"}}))
                .expect("test setup");
            assert_eq!(params["items"][0]["section"], "versionlens");
            let Message::Notification(error) = receive(&client).expect("test setup") else {
                panic!("expected a configuration error message");
            };
            assert_eq!(error.method, "window/showMessage");

            shutdown_client(&client, 2, server_thread).expect("test setup");
        }

        #[test]
        fn a_client_without_configuration_support_gets_defaults_and_no_request() {
            let (client, server_thread) =
                start_workspace_server(json!({"capabilities": {}})).expect("test setup");
            send_notification(
                &client,
                "workspace/didChangeConfiguration",
                json!({"settings": {"versionlens": {"caching": {"duration": "invalid"}}}}),
            )
            .expect("test setup");
            assert!(matches!(
                client.receiver.recv_timeout(Duration::from_millis(500)),
                Err(crossbeam_channel::RecvTimeoutError::Timeout)
            ));
            shutdown_client(&client, 2, server_thread).expect("test setup");
        }

        #[test]
        fn an_answer_to_an_older_settings_request_is_dropped() {
            let (client, server_thread) =
                start_configured_server(json!({}), json!({})).expect("test setup");
            for _ in 0..2 {
                send_notification(&client, "workspace/didChangeConfiguration", json!({}))
                    .expect("test setup");
            }
            let mut requests = Vec::new();
            while requests.len() < 2 {
                if let Message::Request(request) = receive(&client).expect("test setup")
                    && request.method == "workspace/configuration"
                {
                    requests.push(request);
                }
            }
            let answer = |request: &Request, settings: Value| {
                client
                    .sender
                    .send(Message::Response(lsp_server::Response::new_ok(
                        request.id.clone(),
                        json!([settings, null]),
                    )))
            };
            // Both answers are invalid, so the configuration error shows which one
            // was applied: none for the older answer, one for the newest.
            let invalid = json!({"caching": {"duration": "invalid"}});
            answer(&requests[0], invalid.clone()).expect("test setup");
            assert!(
                matches!(
                    receive_within(&client, Duration::from_millis(500)),
                    Err(crossbeam_channel::RecvTimeoutError::Timeout)
                ),
                "the older answer was applied"
            );
            answer(&requests[1], invalid).expect("test setup");
            let Message::Notification(error) = receive(&client).expect("test setup") else {
                panic!("expected the newest answer to be applied");
            };
            assert_eq!(error.method, "window/showMessage");
            shutdown_client(&client, 2, server_thread).expect("test setup");
        }

        /// Receives the next message that is not a log or status notification.
        fn receive_within(
            client: &Connection,
            timeout: Duration,
        ) -> Result<Message, crossbeam_channel::RecvTimeoutError> {
            loop {
                let message = client.receiver.recv_timeout(timeout)?;
                if !matches!(
                    &message,
                    Message::Notification(notification)
                        if matches!(notification.method.as_str(), "window/logMessage" | "versionlens/status")
                ) {
                    return Ok(message);
                }
            }
        }

        #[test]
        fn each_folder_is_presented_with_its_own_settings() {
            let (address, _) = incremental::serving_registry(
                r#"{"dist-tags":{"latest":"2.0.0","next":"3.0.0-beta.1"},"versions":{"1.0.0":{},"2.0.0":{},"3.0.0-beta.1":{}}}"#,
            ).expect("test setup");
            let (server, client) = Connection::memory();
            let server_thread = thread::spawn(move || run_connection(&server, None));
            let (with, without) = ("file:///workspace/with", "file:///workspace/without");
            initialize_workspace(
                &client,
                json!({
                    "workspaceFolders": [
                        {"uri": with, "name": "with"},
                        {"uri": without, "name": "without"}
                    ],
                    "capabilities": {"workspace": {"configuration": true}}
                }),
            )
            .expect("test setup");
            let settings = |prereleases: bool| {
                json!({
                    "suggestions": {"showVulnerabilities": false, "showPrereleasesOnStartup": prereleases},
                    "npm": {"apiUrl": format!("http://{address}")}
                })
            };
            let Message::Request(request) = receive(&client).expect("test setup") else {
                panic!("expected the settings request");
            };
            assert_eq!(request.params["items"][0]["scopeUri"], with);
            assert_eq!(request.params["items"][2]["scopeUri"], without);
            client
                .sender
                .send(Message::Response(lsp_server::Response::new_ok(
                    request.id,
                    json!([settings(true), null, settings(false), null]),
                )))
                .expect("test setup");

            let manifest = r#"{"dependencies": {"alpha": "1.0.0"}}"#;
            let mut titles = Vec::new();
            for (id, folder) in [(10, with), (11, without)] {
                let uri = format!("{folder}/package.json");
                open_json_document(&client, &uri, 1, manifest).expect("test setup");
                titles.push(incremental::lens_titles(&client, id, &uri).expect("test setup"));
            }
            let shows_prerelease =
                |titles: &[String]| titles.iter().any(|title| title.contains("3.0.0-beta.1"));
            assert!(
                shows_prerelease(&titles[0]),
                "with prereleases: {:?}",
                titles[0]
            );
            assert!(
                !shows_prerelease(&titles[1]),
                "without prereleases: {:?}",
                titles[1]
            );
            // Background checking may still publish, so skip to the shutdown answer.
            send_request(&client, 12, "shutdown", Value::Null).expect("test setup");
            while !matches!(receive(&client).expect("test setup"), Message::Response(response) if response.id == 12.into())
            {
            }
            send_notification(&client, "exit", Value::Null).expect("test setup");
            server_thread
                .join()
                .expect("server panicked")
                .expect("test setup");
        }
    }

    #[cfg(unix)]
    mod cache {
        use std::os::unix::fs::PermissionsExt;

        use super::*;

        #[test]
        fn an_unwritable_cache_directory_falls_back_to_memory_with_one_warning() -> Result<()> {
            let parent =
                versionlens_test_support::temporary_directory("versionlens-lsp-read-only-cache")?;
            std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o555))?;
            let directory = parent.join("cache");
            let (server, client) = Connection::memory();
            let server_thread = thread::spawn(move || run_connection(&server, Some(Ok(directory))));

            let result = (|| {
                initialize_workspace(&client, json!({"capabilities": {}}))?;
                send_request(&client, 2, "shutdown", Value::Null)?;
                let mut warnings = Vec::new();
                loop {
                    match client.receiver.recv_timeout(Duration::from_secs(10))? {
                        Message::Notification(notification)
                            if notification.method == "window/logMessage"
                                && notification.params["type"] == 2 =>
                        {
                            warnings.push(notification.params["message"].clone());
                        }
                        Message::Response(response) if response.id == 2.into() => break,
                        _ => {}
                    }
                }
                send_notification(&client, "exit", Value::Null)?;
                server_thread
                    .join()
                    .map_err(|_| anyhow::anyhow!("server panicked"))??;
                anyhow::ensure!(warnings.len() == 1, "warnings: {warnings:?}");
                let warning = warnings[0].as_str().unwrap_or_default();
                anyhow::ensure!(
                    warning.contains("persistent cache is disabled"),
                    "warning: {warning}"
                );
                Ok(())
            })();

            std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755))?;
            std::fs::remove_dir_all(&parent)?;
            result
        }
    }
    #[cfg(test)]
    mod diagnostics {
        use super::*;

        const ASTRO: &str =
            r#"{"dist-tags":{"latest":"7.3.5"},"versions":{"7.2.2":{},"7.3.5":{}}}"#;

        fn start(
            body: &'static str,
            diagnostics: &Value,
        ) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
            let (address, _) = incremental::serving_registry(body)?;
            start_configured_server(
                json!({"capabilities": {}}),
                json!({
                    "suggestions": {"showVulnerabilities": false},
                    "diagnostics": diagnostics,
                    "npm": {"apiUrl": format!("http://{address}")}
                }),
            )
        }

        /// The last diagnostics published for `uri` until checking it is idle, or
        /// an empty list when none were published.
        fn settled_diagnostics(client: &Connection, uri: &str) -> Result<Vec<Value>> {
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            let (mut last, mut busy) = (Vec::new(), false);
            loop {
                let message = client
                    .receiver
                    .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))?;
                let Message::Notification(notification) = message else {
                    continue;
                };
                if notification.params["uri"] != uri {
                    continue;
                }
                match notification.method.as_str() {
                    "textDocument/publishDiagnostics" => {
                        last = notification.params["diagnostics"]
                            .as_array()
                            .cloned()
                            .unwrap_or_default();
                    }
                    "versionlens/status" => match notification.params["status"].as_str() {
                        Some("busy") => busy = true,
                        Some("idle") if busy => return Ok(last),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        fn open(client: &Connection, uri: &str, package: &str, requirement: &str) -> Result<()> {
            open_json_document(
                client,
                uri,
                1,
                &format!(r#"{{"dependencies":{{"{package}":"{requirement}"}}}}"#),
            )
        }

        #[test]
        fn an_outdated_range_gets_an_information_diagnostic_on_its_requirement() {
            let (client, server_thread) = start(ASTRO, &json!({})).expect("test setup");
            let uri = "file:///workspace/astro/package.json";
            open(&client, uri, "astro", "^7.2.2").expect("test setup");

            let diagnostics = settled_diagnostics(&client, uri).expect("test setup");

            assert_eq!(
                diagnostics,
                [json!({
                    "range": {"start": {"line": 0, "character": 26}, "end": {"line": 0, "character": 32}},
                    "severity": 3,
                    "code": "update-available",
                    "source": "VersionLens",
                    "message": "^7.2.2 → ^7.3.5"
                })]
            );
            shutdown_client(&client, 2, server_thread).expect("test setup");
        }

        #[test]
        fn a_patch_only_floor_bump_keeps_its_lens_and_gets_no_diagnostic() {
            let (client, server_thread) = start(
                r#"{"dist-tags":{"latest":"2.5.14"},"versions":{"2.5.8":{},"2.5.14":{}}}"#,
                &json!({}),
            )
            .expect("test setup");
            let uri = "file:///workspace/biome/package.json";
            open(&client, uri, "@biomejs/biome", "^2.5.8").expect("test setup");

            assert_eq!(
                settled_diagnostics(&client, uri).expect("test setup"),
                Vec::<Value>::new()
            );
            let titles = incremental::lens_titles(&client, 2, uri).expect("test setup");
            assert!(
                titles.iter().any(|title| title.contains("2.5.14")),
                "lenses: {titles:?}"
            );
            shutdown(&client, 3, server_thread).expect("test setup");
        }

        #[test]
        fn update_severity_off_publishes_no_update_diagnostics() {
            let (client, server_thread) =
                start(ASTRO, &json!({"updateSeverity": "off"})).expect("test setup");
            let uri = "file:///workspace/off/package.json";
            open(&client, uri, "astro", "^7.2.2").expect("test setup");

            assert_eq!(
                settled_diagnostics(&client, uri).expect("test setup"),
                Vec::<Value>::new()
            );
            shutdown_client(&client, 2, server_thread).expect("test setup");
        }

        #[test]
        fn closing_a_document_clears_its_update_diagnostics() {
            let (client, server_thread) =
                start(ASTRO, &json!({"updateSeverity": "warn"})).expect("test setup");
            let uri = "file:///workspace/close/package.json";
            open(&client, uri, "astro", "^7.2.2").expect("test setup");
            let diagnostics = settled_diagnostics(&client, uri).expect("test setup");
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0]["severity"], 2);

            send_notification(
                &client,
                "textDocument/didClose",
                json!({"textDocument": {"uri": uri}}),
            )
            .expect("test setup");
            loop {
                let Message::Notification(notification) = receive(&client).expect("test setup")
                else {
                    continue;
                };
                if notification.method == "textDocument/publishDiagnostics"
                    && notification.params["uri"] == uri
                {
                    assert_eq!(notification.params["diagnostics"], json!([]));
                    break;
                }
            }
            shutdown_client(&client, 2, server_thread).expect("test setup");
        }

        /// Shuts down after skipping the publishes that a lens request can start.
        fn shutdown(
            client: &Connection,
            id: i32,
            server_thread: thread::JoinHandle<Result<()>>,
        ) -> Result<()> {
            send_request(client, id, "shutdown", Value::Null)?;
            while !matches!(receive(client)?, Message::Response(response) if response.id == id.into())
            {
            }
            send_notification(client, "exit", Value::Null)?;
            server_thread
                .join()
                .map_err(|_| anyhow::anyhow!("server panicked"))?
        }

        #[test]
        fn code_actions_offer_each_bump_inline_without_registry_requests() {
            let (address, requests) = incremental::serving_registry(
                r#"{"dist-tags":{"latest":"8.0.1"},"versions":{"7.2.2":{},"7.3.5":{},"8.0.1":{}}}"#,
            )
            .expect("test setup");
            let (client, server_thread) = start_configured_server(
                json!({"capabilities": {"workspace": {"workspaceEdit": {"documentChanges": true}}}}),
                json!({
                    "suggestions": {"showVulnerabilities": false},
                    "npm": {"apiUrl": format!("http://{address}")}
                }),
            ).expect("test setup");
            let uri = "file:///workspace/actions/package.json";
            open(&client, uri, "astro", "^7.2.2").expect("test setup");
            let diagnostics = settled_diagnostics(&client, uri).expect("test setup");
            let before = requests
                .lock()
                .map(|requests| requests.clone())
                .unwrap_or_default();

            let cursor = json!({"line": 0, "character": 28});
            send_request(
                &client,
                2,
                "textDocument/codeAction",
                json!({
                    "textDocument": {"uri": uri},
                    "range": {"start": cursor, "end": cursor},
                    "context": {"diagnostics": diagnostics}
                }),
            )
            .expect("test setup");
            let actions = loop {
                if let Message::Response(response) = receive(&client).expect("test setup")
                    && response.id == 2.into()
                {
                    break ok_result(Message::Response(response)).expect("test setup");
                }
            };

            let requirement =
                json!({"start": {"line": 0, "character": 26}, "end": {"line": 0, "character": 32}});
            let action = |text: &str, preferred: bool| {
                json!({
                    "title": format!("Bump to {text}"),
                    "kind": "quickfix",
                    "diagnostics": diagnostics,
                    "isPreferred": preferred,
                    "edit": {"documentChanges": [{
                        "textDocument": {"uri": uri, "version": 1},
                        "edits": [{"range": requirement, "newText": text}]
                    }]}
                })
            };
            assert_eq!(
                actions,
                json!([action("^7.3.5", false), action("^8.0.1", true)])
            );
            let after = requests
                .lock()
                .map(|requests| requests.clone())
                .unwrap_or_default();
            assert_eq!(
                before, after,
                "the code action request reached the registry"
            );
            shutdown(&client, 3, server_thread).expect("test setup");
        }
    }

    #[cfg(test)]
    mod commands {
        use super::*;

        const REGISTRY: &str =
            r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"1.1.0":{},"2.0.0":{}}}"#;
        const MANIFEST: &str = r#"{"dependencies":{"beta":"1.0.0","alpha":"1.0.0"}}"#;

        struct Session {
            client: Connection,
            server: thread::JoinHandle<Result<()>>,
            requests: incremental::Requests,
            uri: &'static str,
        }

        fn start(uri: &'static str) -> Result<Session> {
            start_with(uri, MANIFEST)
        }

        fn start_with(uri: &'static str, text: &str) -> Result<Session> {
            let (address, requests) = incremental::serving_registry(REGISTRY)?;
            let (client, server) = start_configured_server(
                json!({"capabilities": {"workspace": {"applyEdit": true, "workspaceEdit": {"documentChanges": true}}}}),
                json!({
                    "suggestions": {"showVulnerabilities": false},
                    "npm": {"apiUrl": format!("http://{address}")}
                }),
            )?;
            open_json_document(&client, uri, 1, text)?;
            Ok(Session {
                client,
                server,
                requests,
                uri,
            })
        }

        impl Session {
            fn execute(&self, id: i32, command: &str, arguments: &Value) -> Result<()> {
                send_request(
                    &self.client,
                    id,
                    "workspace/executeCommand",
                    json!({"command": command, "arguments": [arguments]}),
                )
            }

            /// The next `workspace/applyEdit` request, skipping notifications.
            fn next_edit(&self) -> Result<Request> {
                loop {
                    match receive(&self.client)? {
                        Message::Request(request) if request.method == "workspace/applyEdit" => {
                            return Ok(request);
                        }
                        Message::Notification(_) => {}
                        message => bail!("expected a workspace edit, got {message:?}"),
                    }
                }
            }

            fn answer_edit(&self, request: Request, result: Value) -> Result<()> {
                self.client
                    .sender
                    .send(Message::Response(lsp_server::Response::new_ok(
                        request.id, result,
                    )))?;
                Ok(())
            }

            /// The response to request `id`, skipping notifications.
            fn response(&self, id: i32) -> Result<lsp_server::Response> {
                loop {
                    match receive(&self.client)? {
                        Message::Response(response) if response.id == id.into() => {
                            return Ok(response);
                        }
                        Message::Notification(_) => {}
                        message => bail!("expected response {id}, got {message:?}"),
                    }
                }
            }

            fn registry_requests(&self) -> usize {
                self.requests
                    .lock()
                    .map(|requests| requests.values().sum())
                    .unwrap_or_default()
            }

            fn shutdown(self, id: i32) -> Result<()> {
                send_request(&self.client, id, "shutdown", Value::Null)?;
                self.response(id)?;
                send_notification(&self.client, "exit", Value::Null)?;
                self.server
                    .join()
                    .map_err(|_| anyhow::anyhow!("server panicked"))?
            }
        }

        fn new_texts(request: &Request) -> Vec<String> {
            request.params["edit"]["documentChanges"][0]["edits"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|edit| edit["newText"].as_str().map(ToOwned::to_owned))
                .collect()
        }

        #[test]
        fn bulk_update_applies_one_edit_for_every_dependency_at_the_level() {
            let session = start("file:///workspace/bulk/package.json").expect("test setup");
            session
                .execute(
                    2,
                    "versionlens.server.updateAll",
                    &json!({"uri": session.uri, "level": "minor"}),
                )
                .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            assert_eq!(new_texts(&edit), ["1.1.0", "1.1.0"]);
            session
                .answer_edit(edit, json!({"applied": true}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(2)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            session.shutdown(3).expect("test setup");
        }

        #[test]
        fn update_applies_the_named_dependency_at_its_level() {
            let session = start("file:///workspace/one/package.json").expect("test setup");
            session
                .execute(
                    2,
                    "versionlens.server.update",
                    &json!({"uri": session.uri, "dependency": "alpha", "level": "major"}),
                )
                .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            assert_eq!(new_texts(&edit), ["2.0.0"]);
            assert_eq!(
                edit.params["edit"]["documentChanges"][0]["edits"][0]["range"]["start"]["character"],
                41
            );
            session
                .answer_edit(edit, json!({"applied": true}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(2)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            session.shutdown(3).expect("test setup");
        }

        #[test]
        fn choose_build_applies_the_chosen_version() {
            let session = start("file:///workspace/build/package.json").expect("test setup");
            session
                .execute(
                    2,
                    "versionlens.server.chooseBuild",
                    &json!({"uri": session.uri, "dependency": "beta", "build": "1.1.0"}),
                )
                .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            assert_eq!(new_texts(&edit), ["1.1.0"]);
            session
                .answer_edit(edit, json!({"applied": true}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(2)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            session.shutdown(3).expect("test setup");
        }

        #[test]
        fn sort_applies_the_dependencies_in_name_order() {
            let session = start("file:///workspace/sort/package.json").expect("test setup");
            session
                .execute(2, "versionlens.server.sort", &json!({"uri": session.uri}))
                .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            let texts = new_texts(&edit).concat();
            assert!(
                texts.find("alpha") < texts.find("beta"),
                "sorted text: {texts}"
            );
            session
                .answer_edit(edit, json!({"applied": true}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(2)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            session.shutdown(3).expect("test setup");
        }

        #[test]
        fn a_rejected_edit_returns_the_editor_reason() {
            let session = start("file:///workspace/rejected/package.json").expect("test setup");
            session
                .execute(
                    2,
                    "versionlens.server.updateAll",
                    &json!({"uri": session.uri, "level": "latest"}),
                )
                .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            session
                .answer_edit(
                    edit,
                    json!({"applied": false, "failureReason": "file is read-only"}),
                )
                .expect("test setup");
            let error = session
                .response(2)
                .expect("test setup")
                .response_result
                .err()
                .ok_or_else(|| anyhow::anyhow!("expected an error"))
                .expect("test setup");
            assert_eq!(error.code, ErrorCode::RequestFailed as i32);
            assert!(
                error.message.contains("file is read-only"),
                "{}",
                error.message
            );
            session.shutdown(3).expect("test setup");
        }

        #[test]
        fn an_update_after_an_edit_applies_against_the_current_document() {
            let session = start("file:///workspace/stale/package.json").expect("test setup");
            send_notification(
                &session.client,
                "textDocument/didChange",
                json!({
                    "textDocument": {"uri": session.uri, "version": 2},
                    "contentChanges": [{"text": r#"{"dependencies":{"beta":"1.1.0","alpha":"1.0.0"}}"#}]
                }),
            ).expect("test setup");
            session
                .execute(
                    2,
                    "versionlens.server.update",
                    &json!({"uri": session.uri, "dependency": "beta"}),
                )
                .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            assert_eq!(
                edit.params["edit"]["documentChanges"][0]["textDocument"]["version"],
                2
            );
            assert_eq!(new_texts(&edit), ["2.0.0"]);
            session
                .answer_edit(edit, json!({"applied": true}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(2)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            session.shutdown(3).expect("test setup");
        }

        #[test]
        fn clear_cache_makes_the_next_resolution_fetch_again() {
            let session = start("file:///workspace/cache/package.json").expect("test setup");
            incremental::lens_titles(&session.client, 2, session.uri).expect("test setup");
            let before = session.registry_requests();
            assert!(before > 0);

            session
                .execute(3, "versionlens.server.clearCache", &json!({}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(3)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            incremental::lens_titles(&session.client, 4, session.uri).expect("test setup");

            assert!(
                session.registry_requests() > before,
                "no registry request after clearing the cache"
            );
            session.shutdown(5).expect("test setup");
        }

        #[test]
        fn manifest_info_describes_the_document() {
            // One-line manifests cannot be sorted, so this one has a line per entry.
            let session = start_with(
                "file:///workspace/info/package.json",
                "{\n  \"dependencies\": {\n    \"beta\": \"1.0.0\",\n    \"alpha\": \"1.0.0\"\n  }\n}\n",
            ).expect("test setup");
            session
                .execute(
                    2,
                    "versionlens.server.manifestInfo",
                    &json!({"uri": session.uri}),
                )
                .expect("test setup");

            let info = session
                .response(2)
                .expect("test setup")
                .response_result
                .ok()
                .ok_or_else(|| anyhow::anyhow!("expected a result"))
                .expect("test setup");
            assert_eq!(info["supported"], true);
            assert_eq!(info["provider"], "npm");
            assert_eq!(info["canSortDependencies"], true);
            assert_eq!(info["installTaskConfigKey"], "npm.onSaveChanges");
            assert_eq!(info["dependencyCount"], 2);
            assert!(
                info["dependencySignature"]
                    .as_str()
                    .is_some_and(|signature| !signature.is_empty())
            );
            session.shutdown(3).expect("test setup");
        }

        #[test]
        fn positional_arguments_are_rejected() {
            let session = start("file:///workspace/positional/package.json").expect("test setup");
            send_request(
                &session.client,
                2,
                "workspace/executeCommand",
                json!({"command": "versionlens.server.update", "arguments": [session.uri, "alpha"]}),
            ).expect("test setup");

            let error = session
                .response(2)
                .expect("test setup")
                .response_result
                .err()
                .ok_or_else(|| anyhow::anyhow!("expected an error"))
                .expect("test setup");
            assert_eq!(error.code, ErrorCode::InvalidParams as i32);
            session.shutdown(3).expect("test setup");
        }

        /// The update command of the lens for `dependency`.
        fn lens_command(
            session: &Session,
            id: i32,
            dependency: &str,
        ) -> Result<lsp_types::Command> {
            send_request(
                &session.client,
                id,
                "textDocument/codeLens",
                json!({"textDocument": {"uri": session.uri}}),
            )?;
            let lenses = session
                .response(id)?
                .response_result
                .map_err(|error| anyhow::anyhow!("code lens request failed: {error:?}"))?;
            serde_json::from_value::<Vec<CodeLens>>(lenses)?
                .into_iter()
                .filter_map(|lens| lens.command)
                .find(|command| {
                    command.command == "versionlens.server.update"
                        && command.arguments.as_ref().is_some_and(|arguments| {
                            arguments[0]["dependency"]
                                .as_str()
                                .is_some_and(|selector| selector.starts_with(dependency))
                        })
                })
                .ok_or_else(|| anyhow::anyhow!("no update lens for {dependency}"))
        }

        #[test]
        fn a_lens_clicked_after_its_line_moved_updates_the_current_entry() {
            let session = start("file:///workspace/moved/package.json").expect("test setup");
            let command = lens_command(&session, 2, "beta").expect("test setup");
            send_notification(
                &session.client,
                "textDocument/didChange",
                json!({
                    "textDocument": {"uri": session.uri, "version": 2},
                    "contentChanges": [{"text": format!("\n{MANIFEST}")}]
                }),
            )
            .expect("test setup");
            send_request(
                &session.client,
                3,
                "workspace/executeCommand",
                serde_json::to_value(&command).expect("test setup"),
            )
            .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            let change = &edit.params["edit"]["documentChanges"][0];
            assert_eq!(change["textDocument"]["version"], 2);
            assert_eq!(change["edits"][0]["range"]["start"]["line"], 1);
            session
                .answer_edit(edit, json!({"applied": true}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(3)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            session.shutdown(4).expect("test setup");
        }

        #[test]
        fn an_update_with_a_version_and_no_level_applies_that_version() {
            let session = start("file:///workspace/version/package.json").expect("test setup");
            session
                .execute(
                    2,
                    "versionlens.server.update",
                    &json!({"uri": session.uri, "dependency": "alpha", "version": "1.1.0"}),
                )
                .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            assert_eq!(new_texts(&edit), ["1.1.0"]);
            session
                .answer_edit(edit, json!({"applied": true}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(2)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            session.shutdown(3).expect("test setup");
        }

        fn workspace_root(name: &str) -> Result<(std::path::PathBuf, String)> {
            let root = versionlens_test_support::temporary_directory(name)?.canonicalize()?;
            std::fs::write(root.join(".gitignore"), "skipped/\n")?;
            for directory in ["first", "second", "skipped"] {
                std::fs::create_dir_all(root.join(directory))?;
                std::fs::write(root.join(directory).join("package.json"), MANIFEST)?;
            }
            let root_uri =
                versionlens_engine::workspace_file_uri(&root).context("missing root URI")?;
            Ok((root, root_uri))
        }

        fn start_workspace_update_server(
            root_uri: &str,
        ) -> Result<(Session, std::net::SocketAddr)> {
            let (address, requests) = incremental::serving_registry(REGISTRY)?;
            let (client, server) = start_configured_server(
                json!({"rootUri": root_uri, "capabilities": {"workspace": {"applyEdit": true, "workspaceEdit": {"documentChanges": true}}}}),
                json!({
                    "suggestions": {"showVulnerabilities": false},
                    "npm": {"apiUrl": format!("http://{address}")}
                }),
            )?;
            let session = Session {
                client,
                server,
                requests,
                uri: "",
            };
            Ok((session, address))
        }

        /// The folder and joined new texts of each manifest a workspace edit changes.
        fn changed_manifests(edit: &Request) -> Vec<(String, String)> {
            let mut changed = edit.params["edit"]["documentChanges"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|change| {
                    let uri = change["textDocument"]["uri"].as_str().unwrap_or_default();
                    let texts = change["edits"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|edit| edit["newText"].as_str())
                        .collect::<Vec<_>>();
                    (
                        uri.rsplit('/').nth(1).unwrap_or_default().to_owned(),
                        texts.join(","),
                    )
                })
                .collect::<Vec<_>>();
            changed.sort();
            changed
        }

        #[test]
        fn workspace_update_edits_every_unignored_manifest_without_opening_it() {
            let (root, root_uri) =
                workspace_root("versionlens-lsp-update-workspace").expect("test setup");
            let (session, _) = start_workspace_update_server(&root_uri).expect("test setup");
            session
                .execute(
                    2,
                    "versionlens.server.updateWorkspace",
                    &json!({"level": "minor"}),
                )
                .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            assert_eq!(
                changed_manifests(&edit),
                [
                    ("first".to_owned(), "1.1.0,1.1.0".to_owned()),
                    ("second".to_owned(), "1.1.0,1.1.0".to_owned())
                ]
            );
            session
                .answer_edit(edit, json!({"applied": true}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(2)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            session.shutdown(3).expect("test setup");
            std::fs::remove_dir_all(root).expect("test setup");
        }

        #[test]
        fn workspace_update_waits_for_pending_settings() {
            let (root, root_uri) =
                workspace_root("versionlens-lsp-update-workspace-settings").expect("test setup");
            let (session, address) = start_workspace_update_server(&root_uri).expect("test setup");
            send_notification(
                &session.client,
                "workspace/didChangeConfiguration",
                json!({}),
            )
            .expect("test setup");
            session
                .execute(
                    2,
                    "versionlens.server.updateWorkspace",
                    &json!({"level": "minor"}),
                )
                .expect("test setup");
            answer_configuration(
                &session.client,
                json!({
                    "suggestions": {"showVulnerabilities": false},
                    "npm": {"apiUrl": format!("http://{address}")},
                    "workspace": {"exclude": ["**/second/**"]}
                }),
            )
            .expect("test setup");

            let edit = session.next_edit().expect("test setup");
            assert_eq!(
                changed_manifests(&edit),
                [("first".to_owned(), "1.1.0,1.1.0".to_owned())]
            );
            session
                .answer_edit(edit, json!({"applied": true}))
                .expect("test setup");
            assert_eq!(
                session
                    .response(2)
                    .expect("test setup")
                    .response_result
                    .ok(),
                Some(Value::Null)
            );
            session.shutdown(3).expect("test setup");
            std::fs::remove_dir_all(root).expect("test setup");
        }
    }

    mod authorization {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::{Arc, Mutex};
        use std::time::Instant;

        use lsp_server::{RequestId, Response};

        use super::*;
        use crate::server::run_connection_with;

        const TOKEN: &str = "secret-token";
        const RESPONSE_BUDGET: Duration = Duration::from_millis(500);

        /// A registry that answers 401 until a request carries the token.
        struct Registry {
            address: std::net::SocketAddr,
            /// The `Authorization` header of each request, in arrival order.
            requests: Arc<Mutex<Vec<Option<String>>>>,
            stop: Arc<AtomicBool>,
        }

        impl Registry {
            fn start() -> Result<Self> {
                Self::serve(false)
            }

            /// A registry that answers every request, with or without the token.
            fn open() -> Result<Self> {
                Self::serve(true)
            }

            fn serve(open: bool) -> Result<Self> {
                let listener = TcpListener::bind("127.0.0.1:0")?;
                listener.set_nonblocking(true)?;
                let address = listener.local_addr()?;
                let requests = Arc::new(Mutex::new(Vec::new()));
                let stop = Arc::new(AtomicBool::new(false));
                let (log, flag) = (Arc::clone(&requests), Arc::clone(&stop));
                // Detached: it stops on its own once `stop` is set.
                let _server = thread::spawn(move || {
                    while !flag.load(Ordering::Acquire) {
                        let Ok((socket, _)) = listener.accept() else {
                            thread::sleep(Duration::from_millis(10));
                            continue;
                        };
                        let log = Arc::clone(&log);
                        thread::spawn(move || {
                            drop(answer(socket, &log, open));
                        });
                    }
                });
                Ok(Self {
                    address,
                    requests,
                    stop,
                })
            }

            fn url(&self) -> String {
                format!("http://{}", self.address)
            }

            fn headers(&self) -> Vec<Option<String>> {
                self.requests
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone()
            }

            fn authorized(&self) -> usize {
                let expected = format!("Bearer {TOKEN}");
                self.headers()
                    .iter()
                    .filter(|header| header.as_deref() == Some(expected.as_str()))
                    .count()
            }
        }

        impl Drop for Registry {
            fn drop(&mut self) {
                self.stop.store(true, Ordering::Release);
            }
        }

        fn answer(
            mut socket: std::net::TcpStream,
            log: &Mutex<Vec<Option<String>>>,
            open: bool,
        ) -> Result<()> {
            socket.set_nonblocking(false)?;
            socket.set_read_timeout(Some(Duration::from_secs(3)))?;
            let mut buffer = [0; 8192];
            let read = socket.read(&mut buffer)?;
            let request = String::from_utf8_lossy(&buffer[..read]).into_owned();
            let header = request.lines().find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("authorization")
                    .then(|| value.trim().to_owned())
            });
            log.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(header.clone());
            if open || header.as_deref() == Some(format!("Bearer {TOKEN}").as_str()) {
                let body = r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#;
                write!(
                    socket,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )?;
            } else {
                write!(
                    socket,
                    "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )?;
            }
            Ok(())
        }

        /// What the client does with `versionlens/authorization`.
        enum Reply {
            Token,
            MethodNotFound,
            /// Never answers.
            Silent,
        }

        fn reply_to(request: Request, reply: &Reply) -> Option<Response> {
            if request.method != "versionlens/authorization" {
                return Some(Response::new_ok(request.id, Value::Null));
            }
            match reply {
                Reply::Token => Some(Response::new_ok(
                    request.id,
                    json!({"scheme":"Bearer","token":TOKEN}),
                )),
                Reply::MethodNotFound => Some(Response::new_err(
                    request.id,
                    ErrorCode::MethodNotFound as i32,
                    "method not found: versionlens/authorization".to_owned(),
                )),
                Reply::Silent => None,
            }
        }

        /// Answers the server's requests for `duration` and returns the
        /// `versionlens/authorization` params it received.
        fn serve(client: &Connection, duration: Duration, reply: &Reply) -> Result<Vec<Value>> {
            let deadline = Instant::now() + duration;
            let mut asked = Vec::new();
            while let Ok(message) = client.receiver.recv_deadline(deadline) {
                let Message::Request(request) = message else {
                    continue;
                };
                if request.method == "versionlens/authorization" {
                    asked.push(request.params.clone());
                }
                if let Some(response) = reply_to(request, reply) {
                    client.sender.send(Message::Response(response))?;
                }
            }
            Ok(asked)
        }

        fn code_lens_titles(
            client: &Connection,
            id: i32,
            uri: &str,
            reply: &Reply,
        ) -> Result<String> {
            send_request(
                client,
                id,
                "textDocument/codeLens",
                json!({"textDocument":{"uri":uri}}),
            )?;
            let id = RequestId::from(id);
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                let message = client.receiver.recv_deadline(deadline)?;
                match message {
                    Message::Response(response) if response.id == id => {
                        let lenses = serde_json::from_value::<Vec<CodeLens>>(
                            response
                                .response_result
                                .map_err(|error| anyhow::anyhow!("{error:?}"))?,
                        )?;
                        return Ok(lenses
                            .iter()
                            .filter_map(|lens| lens.command.as_ref())
                            .map(|command| command.title.as_str())
                            .collect::<Vec<_>>()
                            .join("\n"));
                    }
                    Message::Request(request) => {
                        if let Some(response) = reply_to(request, reply) {
                            client.sender.send(Message::Response(response))?;
                        }
                    }
                    _ => {}
                }
            }
        }

        fn start(registry: &Registry) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
            start_with(registry, authorization_timeout())
        }

        fn authorization_timeout() -> Duration {
            Duration::from_secs(60)
        }

        fn start_with(
            registry: &Registry,
            timeout: Duration,
        ) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
            start_settings(
                json!({"suggestions":{"showVulnerabilities":false},"npm":{"apiUrl":registry.url()}}),
                timeout,
            )
        }

        fn start_settings(
            settings: Value,
            timeout: Duration,
        ) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
            let (server, client) = Connection::memory();
            let server_thread = thread::spawn(move || run_connection_with(&server, None, timeout));
            initialize_configured(
                &client,
                json!({"capabilities":{"workspace":{"codeLens":{"refreshSupport":true}}}}),
                settings,
            )?;
            Ok((client, server_thread))
        }

        #[test]
        fn a_credential_from_the_client_is_sent_on_the_retry() {
            let registry = Registry::start().expect("test setup");
            let (client, server_thread) = start(&registry).expect("test setup");
            let uri = "file:///workspace/package.json";
            open_json_document(
                &client,
                uri,
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )
            .expect("test setup");

            let asked = serve(&client, Duration::from_secs(2), &Reply::Token).expect("test setup");

            assert_eq!(asked, [json!({"host":"127.0.0.1","status":401})]);
            assert_eq!(registry.headers().first(), Some(&None));
            assert!(registry.authorized() >= 1, "{:?}", registry.headers());
            let titles = code_lens_titles(&client, 10, uri, &Reply::Token).expect("test setup");
            assert!(titles.contains("2.0.0"), "{titles}");
            shut_down(&client, 11, server_thread);
        }

        #[test]
        fn a_client_without_the_method_falls_back_and_is_not_asked_again() {
            let registry = Registry::start().expect("test setup");
            let (client, server_thread) = start(&registry).expect("test setup");
            let first = "file:///workspace/a/package.json";
            let second = "file:///workspace/b/package.json";
            let other = "file:///workspace/c/jsr.json";
            open_json_document(
                &client,
                first,
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )
            .expect("test setup");
            let mut asked =
                serve(&client, Duration::from_secs(1), &Reply::MethodNotFound).expect("test setup");
            open_json_document(
                &client,
                second,
                1,
                "{\"dependencies\":{\"another\":\"1.0.0\"}}",
            )
            .expect("test setup");
            open_json_document(
                &client,
                other,
                1,
                "{\"name\":\"@scope/pkg\",\"version\":\"1.0.0\"}",
            )
            .expect("test setup");
            asked.extend(
                serve(&client, Duration::from_secs(1), &Reply::MethodNotFound).expect("test setup"),
            );

            assert_eq!(asked.len(), 1, "{asked:?}");
            assert_eq!(asked[0]["host"], "127.0.0.1");
            assert_eq!(registry.authorized(), 0);
            // Both documents reached the registry, unauthenticated, and none waited.
            assert!(registry.headers().len() >= 2, "{:?}", registry.headers());

            let started = Instant::now();
            code_lens_titles(&client, 10, other, &Reply::MethodNotFound).expect("test setup");
            assert!(
                started.elapsed() < RESPONSE_BUDGET,
                "{:?}",
                started.elapsed()
            );
            shut_down(&client, 11, server_thread);
        }

        /// Shuts the server down, answering its requests and skipping notifications,
        /// and checks that it does so within the budget.
        fn shut_down(client: &Connection, id: i32, server: thread::JoinHandle<Result<()>>) {
            send_request(client, id, "shutdown", Value::Null).expect("test setup");
            let started = Instant::now();
            loop {
                match client
                    .receiver
                    .recv_timeout(RESPONSE_BUDGET)
                    .expect("test setup")
                {
                    Message::Response(response) if response.id == RequestId::from(id) => break,
                    Message::Request(request) => client
                        .sender
                        .send(Message::Response(Response::new_ok(request.id, Value::Null)))
                        .expect("test setup"),
                    _ => {}
                }
            }
            assert!(started.elapsed() < RESPONSE_BUDGET);
            send_notification(client, "exit", Value::Null).expect("test setup");
            server.join().expect("server panicked").expect("test setup");
        }

        fn dependencies(count: usize, prefix: &str) -> String {
            let entries = (0..count)
                .map(|index| format!("\"{prefix}{index}\":\"1.0.0\""))
                .collect::<Vec<_>>()
                .join(",");
            format!("{{\"dependencies\":{{{entries}}}}}")
        }

        #[test]
        fn an_unanswered_question_does_not_block_other_hosts_or_shutdown() {
            let stuck = Registry::start().expect("test setup");
            let other = Registry::open().expect("test setup");
            let (client, server_thread) = start_settings(
                json!({
                    "suggestions":{"showVulnerabilities":false},
                    "npm":{"apiUrl":stuck.url()},
                    "cargo":{"apiUrl":other.url()},
                }),
                authorization_timeout(),
            )
            .expect("test setup");
            // More refused requests than there are registry permits.
            open_json_document(
                &client,
                "file:///workspace/stuck/package.json",
                1,
                &dependencies(24, "stuck"),
            )
            .expect("test setup");
            let asked = serve(&client, Duration::from_secs(1), &Reply::Silent).expect("test setup");
            assert_eq!(asked.len(), 1, "{asked:?}");

            send_notification(
                &client,
                "textDocument/didOpen",
                json!({"textDocument": {
                    "uri": "file:///workspace/other/Cargo.toml",
                    "languageId": "toml",
                    "version": 1,
                    "text": "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1.0.0\"\n",
                }}),
            ).expect("test setup");
            serve(&client, Duration::from_secs(2), &Reply::Silent).expect("test setup");
            assert!(!other.headers().is_empty(), "the other host was not asked");

            shut_down(&client, 20, server_thread);
        }

        #[test]
        fn documents_on_one_host_share_one_question() {
            let registry = Registry::start().expect("test setup");
            let (client, server_thread) = start(&registry).expect("test setup");
            for (index, directory) in ["a", "b", "c"].into_iter().enumerate() {
                open_json_document(
                    &client,
                    &format!("file:///workspace/{directory}/package.json"),
                    1,
                    &dependencies(4, &format!("dep{index}x")),
                )
                .expect("test setup");
            }
            let asked = serve(&client, Duration::from_secs(2), &Reply::Token).expect("test setup");
            assert_eq!(asked.len(), 1, "{asked:?}");
            assert!(registry.authorized() >= 12, "{:?}", registry.headers());
            shut_down(&client, 20, server_thread);
        }

        #[test]
        fn an_answer_after_the_timeout_checks_the_dependency_again() {
            let registry = Registry::start().expect("test setup");
            let (client, server_thread) =
                start_with(&registry, Duration::from_millis(200)).expect("test setup");
            let uri = "file:///workspace/package.json";
            open_json_document(
                &client,
                uri,
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )
            .expect("test setup");
            // Hold the question past the timeout, then answer it.
            let question = loop {
                match client
                    .receiver
                    .recv_timeout(Duration::from_secs(5))
                    .expect("test setup")
                {
                    Message::Request(request) if request.method == "versionlens/authorization" => {
                        break request;
                    }
                    Message::Request(request) => client
                        .sender
                        .send(Message::Response(Response::new_ok(request.id, Value::Null)))
                        .expect("test setup"),
                    _ => {}
                }
            };
            serve(&client, Duration::from_millis(600), &Reply::Silent).expect("test setup");
            assert_eq!(registry.authorized(), 0);
            client
                .sender
                .send(Message::Response(Response::new_ok(
                    question.id,
                    json!({"scheme":"Bearer","token":TOKEN}),
                )))
                .expect("test setup");
            serve(&client, Duration::from_secs(4), &Reply::Silent).expect("test setup");
            assert!(registry.authorized() >= 1, "{:?}", registry.headers());
            let titles = code_lens_titles(&client, 10, uri, &Reply::Silent).expect("test setup");
            assert!(titles.contains("2.0.0"), "{titles}");
            shut_down(&client, 20, server_thread);
        }

        #[test]
        fn a_question_unanswered_at_the_timeout_is_not_asked_again() {
            let registry = Registry::start().expect("test setup");
            let (client, server_thread) =
                start_with(&registry, Duration::from_millis(200)).expect("test setup");
            open_json_document(
                &client,
                "file:///workspace/a/package.json",
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )
            .expect("test setup");
            let mut asked =
                serve(&client, Duration::from_secs(1), &Reply::Silent).expect("test setup");
            open_json_document(
                &client,
                "file:///workspace/b/package.json",
                1,
                "{\"dependencies\":{\"another\":\"1.0.0\"}}",
            )
            .expect("test setup");
            asked.extend(
                serve(&client, Duration::from_secs(1), &Reply::Silent).expect("test setup"),
            );
            assert_eq!(asked.len(), 1, "{asked:?}");
            shut_down(&client, 20, server_thread);
        }
    }

    mod rate_limit {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        use std::time::{Instant, SystemTime, UNIX_EPOCH};

        use lsp_server::{RequestId, Response};

        use super::*;
        use crate::server::run_connection_with;

        /// A registry whose rate limit is exhausted: every request is a 403 with
        /// `x-ratelimit-remaining: 0`.
        struct Exhausted {
            address: std::net::SocketAddr,
            requests: Arc<AtomicUsize>,
            stop: Arc<AtomicBool>,
        }

        impl Exhausted {
            fn start(reset: u64) -> Result<Self> {
                let listener = TcpListener::bind("127.0.0.1:0")?;
                listener.set_nonblocking(true)?;
                let address = listener.local_addr()?;
                let requests = Arc::new(AtomicUsize::new(0));
                let stop = Arc::new(AtomicBool::new(false));
                let (count, flag) = (Arc::clone(&requests), Arc::clone(&stop));
                // Detached: it stops on its own once `stop` is set.
                let _server = thread::spawn(move || {
                    while !flag.load(Ordering::Acquire) {
                        let Ok((mut socket, _)) = listener.accept() else {
                            thread::sleep(Duration::from_millis(10));
                            continue;
                        };
                        count.fetch_add(1, Ordering::AcqRel);
                        thread::spawn(move || {
                            drop(socket.set_nonblocking(false));
                            drop(socket.set_read_timeout(Some(Duration::from_secs(3))));
                            drop(socket.read(&mut [0; 8192]));
                            drop(write!(
                                socket,
                                "HTTP/1.1 403 Forbidden\r\nx-ratelimit-remaining: 0\r\n\
                         x-ratelimit-reset: {reset}\r\nContent-Length: 0\r\n\
                         Connection: close\r\n\r\n"
                            ));
                        });
                    }
                });
                Ok(Self {
                    address,
                    requests,
                    stop,
                })
            }

            fn requests(&self) -> usize {
                self.requests.load(Ordering::Acquire)
            }
        }

        impl Drop for Exhausted {
            fn drop(&mut self) {
                self.stop.store(true, Ordering::Release);
            }
        }

        /// What the client saw while it answered the server's requests.
        #[derive(Default)]
        struct Seen {
            warnings: Vec<String>,
            error_diagnostics: usize,
            authorization_requests: usize,
        }

        fn drain(client: &Connection, duration: Duration, seen: &mut Seen) {
            let deadline = Instant::now() + duration;
            while let Ok(message) = client.receiver.recv_deadline(deadline) {
                match message {
                    Message::Request(request) => {
                        if request.method == "versionlens/authorization" {
                            seen.authorization_requests += 1;
                        }
                        client
                            .sender
                            .send(Message::Response(Response::new_ok(request.id, Value::Null)))
                            .expect("test setup");
                    }
                    Message::Notification(notification) => match notification.method.as_str() {
                        "window/showMessage" => {
                            let params: lsp_types::ShowMessageParams =
                                serde_json::from_value(notification.params).expect("test setup");
                            assert_eq!(params.typ, lsp_types::MessageType::WARNING);
                            seen.warnings.push(params.message);
                        }
                        "textDocument/publishDiagnostics" => {
                            let params: PublishDiagnosticsParams =
                                serde_json::from_value(notification.params).expect("test setup");
                            seen.error_diagnostics += params
                                .diagnostics
                                .iter()
                                .filter(|diagnostic| {
                                    diagnostic.severity
                                        == Some(lsp_types::DiagnosticSeverity::ERROR)
                                })
                                .count();
                        }
                        _ => {}
                    },
                    Message::Response(_) => {}
                }
            }
        }

        fn lens_titles(client: &Connection, id: i32, uri: &str) -> Result<String> {
            send_request(
                client,
                id,
                "textDocument/codeLens",
                json!({"textDocument":{"uri":uri}}),
            )?;
            let id = RequestId::from(id);
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                match client.receiver.recv_deadline(deadline)? {
                    Message::Response(response) if response.id == id => {
                        let lenses = serde_json::from_value::<Vec<CodeLens>>(
                            response
                                .response_result
                                .map_err(|error| anyhow::anyhow!("{error:?}"))?,
                        )?;
                        return Ok(lenses
                            .iter()
                            .filter_map(|lens| lens.command.as_ref())
                            .map(|command| command.title.as_str())
                            .collect::<Vec<_>>()
                            .join("\n"));
                    }
                    Message::Request(request) => client
                        .sender
                        .send(Message::Response(Response::new_ok(request.id, Value::Null)))?,
                    _ => {}
                }
            }
        }

        #[test]
        fn an_exhausted_rate_limit_warns_once_and_stops_requests_until_the_reset() {
            let reset = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test setup")
                .as_secs()
                + 3600;
            let registry = Exhausted::start(reset).expect("test setup");
            let (server, client) = Connection::memory();
            let server_thread =
                thread::spawn(move || run_connection_with(&server, None, Duration::from_secs(60)));
            initialize_configured(
                &client,
                json!({"capabilities":{"workspace":{"codeLens":{"refreshSupport":true}}}}),
                json!({
                    "suggestions":{"showVulnerabilities":false},
                    "npm":{"apiUrl":format!("http://{}", registry.address)},
                }),
            )
            .expect("test setup");
            let uri = "file:///workspace/package.json";
            open_json_document(
                &client,
                uri,
                1,
                "{\"dependencies\":{\"example\":\"1.0.0\"}}",
            )
            .expect("test setup");
            let mut seen = Seen::default();
            // Wait for the warning, which follows the first request, then let any
            // stray request show up before counting.
            let deadline = Instant::now() + Duration::from_secs(15);
            while seen.warnings.is_empty() && Instant::now() < deadline {
                drain(&client, Duration::from_millis(100), &mut seen);
            }
            drain(&client, Duration::from_millis(300), &mut seen);
            assert_eq!(registry.requests(), 1);

            // Another edit and another document, before the reset.
            send_notification(
                &client,
                "textDocument/didChange",
                json!({
                    "textDocument": {"uri": uri, "version": 2},
                    "contentChanges": [{"text": "{\"dependencies\":{\"example\":\"1.1.0\"}}"}]
                }),
            )
            .expect("test setup");
            open_json_document(
                &client,
                "file:///workspace/other/package.json",
                1,
                "{\"dependencies\":{\"another\":\"1.0.0\"}}",
            )
            .expect("test setup");
            drain(&client, Duration::from_secs(2), &mut seen);

            assert_eq!(registry.requests(), 1, "a request was sent while held");
            assert_eq!(seen.warnings.len(), 1, "{:?}", seen.warnings);
            assert!(seen.warnings[0].contains("UTC"), "{:?}", seen.warnings);
            assert_eq!(seen.error_diagnostics, 0);
            assert_eq!(seen.authorization_requests, 0);
            let titles = lens_titles(&client, 10, uri).expect("test setup");
            assert!(titles.contains("rate limited until"), "{titles}");

            send_request(&client, 11, "shutdown", Value::Null).expect("test setup");
            drain(&client, Duration::from_millis(300), &mut seen);
            send_notification(&client, "exit", Value::Null).expect("test setup");
            server_thread
                .join()
                .expect("server panicked")
                .expect("test setup");
        }
    }
}
