use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request};
use lsp_types::{CodeLens, NumberOrString, PublishDiagnosticsParams, ServerCapabilities};
use serde_json::{Value, json};

use super::run_connection;
use crate::state::DISPLAY_CODE_LENS_COMMAND;

#[test]
fn raw_loop_rejects_bad_requests_and_survives_bad_notifications() -> Result<()> {
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, None));

    send_request(&client, 1, "initialize", Value::String("bad".to_owned()))?;
    assert_error(receive(&client)?, ErrorCode::InvalidParams)?;

    initialize_client(&client)?;

    send_request(&client, 3, "textDocument/codeLens", json!({"bad": true}))?;
    assert_error(receive(&client)?, ErrorCode::InvalidParams)?;

    send_request(&client, 4, "versionlens/unknown", json!({}))?;
    assert_error(receive(&client)?, ErrorCode::MethodNotFound)?;

    send_notification(&client, "textDocument/didOpen", Value::Bool(false))?;
    send_notification(&client, "textDocument/didChange", json!({"bad": true}))?;
    send_notification(&client, "versionlens/unknown", json!({"bad": true}))?;

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
    )?;
    assert_diagnostics(receive(&client)?, uri, true)?;

    send_notification(
        &client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri, "version": 2},
            "contentChanges": []
        }),
    )?;
    send_notification(
        &client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri, "version": 3},
            "contentChanges": [{"text": "{\"name\":\"@scope/pkg\",\"version\":\"1.1.0\"}"}]
        }),
    )?;
    assert_diagnostics(receive(&client)?, uri, true)?;

    assert_visible_code_lenses(&client, uri)?;

    send_notification(
        &client,
        "textDocument/didClose",
        json!({"textDocument": {"uri": uri}}),
    )?;
    assert_diagnostics(receive(&client)?, uri, true)?;

    send_request(
        &client,
        8,
        "textDocument/codeLens",
        json!({"textDocument": {"uri": uri}}),
    )?;
    let lenses = serde_json::from_value::<Vec<CodeLens>>(ok_result(receive(&client)?)?)?;
    assert!(lenses.is_empty());

    send_request(&client, 9, "shutdown", Value::Null)?;
    let _shutdown = ok_result(receive(&client)?)?;
    send_notification(&client, "exit", Value::Null)?;

    let server_result = server_thread.join();
    assert!(server_result.is_ok());
    if let Ok(result) = server_result {
        result?;
    }
    Ok(())
}

#[test]
fn dynamically_registers_workspace_file_watching_when_supported() -> Result<()> {
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
    )?;
    let Message::Request(registration) = receive(&client)? else {
        bail!("expected dynamic file watcher registration");
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
        )))?;
    shutdown_client(&client, 2, server_thread)
}

#[test]
fn publishes_unversioned_results_for_unopened_workspace_documents() -> Result<()> {
    let root = versionlens_test_support::temporary_directory("versionlens-lsp-unopened")?;
    let workflow_directory = root.join(".github/workflows");
    std::fs::create_dir_all(&workflow_directory)?;
    let workflow = workflow_directory.join("ci.yml");
    std::fs::write(&workflow, "jobs: {check: {steps: [{uses: './missing'}]}}\n")?;
    let root_uri = versionlens_engine::workspace_file_uri(&root).context("missing root URI")?;
    let workflow_uri = versionlens_engine::workspace_file_uri(&workflow.canonicalize()?)
        .context("missing workflow URI")?;
    let (client, server_thread) = start_configured_server(
        json!({"rootUri": root_uri, "capabilities": {}}),
        json!({"suggestions": {"showVulnerabilities": false}, "workspace": {"check": "workspace"}}),
    )?;
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let message = client
            .receiver
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))?;
        let Message::Notification(notification) = message else {
            continue;
        };
        if notification.method != "textDocument/publishDiagnostics" {
            continue;
        }
        let params = serde_json::from_value::<PublishDiagnosticsParams>(notification.params)?;
        if params.uri.as_str() == workflow_uri {
            assert!(params.diagnostics.is_empty());
            assert_eq!(params.version, None);
            break;
        }
    }
    shutdown_client(&client, 2, server_thread)?;
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn discovery_failures_are_logged_once_and_never_published() -> Result<()> {
    let root = versionlens_test_support::temporary_directory("versionlens-lsp-retry")?;
    std::fs::remove_dir_all(&root)?;
    let root_uri = versionlens_engine::workspace_file_uri(&root).context("missing root URI")?;
    let (client, server_thread) = start_configured_server(
        json!({"rootUri": root_uri, "capabilities": {}}),
        json!({"workspace": {"check": "workspace"}}),
    )?;

    // The missing root fails discovery, which retries after 1 s; the retry
    // fails the same way and must not be logged again.
    let mut logged = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_millis(2500);
    while let Some(wait) = deadline.checked_duration_since(std::time::Instant::now()) {
        let Ok(message) = client.receiver.recv_timeout(wait) else {
            break;
        };
        let Message::Notification(notification) = message else {
            bail!("unexpected message {message:?}");
        };
        match notification.method.as_str() {
            "window/logMessage" => logged.push(notification.params),
            "versionlens/status" => {}
            method => bail!("unexpected notification {method}"),
        }
    }
    assert_eq!(logged.len(), 1, "{logged:?}");
    assert_eq!(logged[0]["type"], 2);

    shutdown_client(&client, 2, server_thread)?;
    Ok(())
}

fn initialize_client(client: &Connection) -> Result<()> {
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
    )?;
    let initialize_result = ok_result(receive(client)?)?;
    let capabilities =
        serde_json::from_value::<ServerCapabilities>(initialize_result["capabilities"].clone())?;
    assert!(capabilities.code_lens_provider.is_some());
    assert!(capabilities.code_action_provider.is_some());
    let execute_commands = capabilities
        .execute_command_provider
        .ok_or_else(|| anyhow::anyhow!("expected execute command capabilities"))?;
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
    send_notification(client, "initialized", json!({}))
}

fn initialize_workspace(client: &Connection, params: Value) -> Result<Value> {
    send_request(client, 1, "initialize", params)?;
    let result = ok_result(receive(client)?)?;
    send_notification(client, "initialized", json!({}))?;
    Ok(result)
}

/// Initializes a client that answers `workspace/configuration` with
/// `settings` for the `versionlens` section and `null` for the others.
fn initialize_configured(client: &Connection, mut params: Value, settings: Value) -> Result<Value> {
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

fn start_workspace_server(params: Value) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, None));
    initialize_workspace(&client, params)?;
    Ok((client, server_thread))
}

fn assert_visible_code_lenses(client: &Connection, uri: &str) -> Result<()> {
    send_request(
        client,
        5,
        "textDocument/codeLens",
        json!({"textDocument": {"uri": uri}}),
    )?;
    let lenses = serde_json::from_value::<Vec<CodeLens>>(ok_result(receive(client)?)?)?;
    crate::test_support::assert_update_code_lenses(&lenses);
    assert_diagnostics(receive(client)?, uri, true)?;

    send_request(
        client,
        6,
        "workspace/executeCommand",
        json!({"command": DISPLAY_CODE_LENS_COMMAND}),
    )?;
    assert_eq!(ok_result(receive(client)?)?, Value::Null);

    send_request(
        client,
        7,
        "workspace/executeCommand",
        json!({"command": "versionlens.update", "arguments": []}),
    )?;
    assert_error(receive(client)?, ErrorCode::InvalidParams)
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
    assert_diagnostics(receive(client)?, uri, true)?;
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

fn assert_error(message: Message, expected_code: ErrorCode) -> Result<()> {
    let Message::Response(response) = message else {
        bail!("expected response, got {message:?}");
    };
    let error = response
        .response_result
        .expect_err("expected error response");
    assert_eq!(error.code, expected_code as i32);
    Ok(())
}

/// Asserts a diagnostics notification for `expected_uri`, and whether it has
/// no diagnostic other than an available update.
fn assert_diagnostics(message: Message, expected_uri: &str, empty: bool) -> Result<()> {
    let Message::Notification(notification) = message else {
        bail!("expected diagnostics notification, got {message:?}");
    };
    assert_eq!(notification.method, "textDocument/publishDiagnostics");
    let params = serde_json::from_value::<PublishDiagnosticsParams>(notification.params)?;
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
    Ok(())
}

mod updates;

mod refresh;

mod responsiveness;

mod incremental;

mod push;

mod configuration;

#[cfg(unix)]
mod cache;
#[cfg(test)]
mod diagnostics {
use super::*;

const ASTRO: &str = r#"{"dist-tags":{"latest":"7.3.5"},"versions":{"7.2.2":{},"7.3.5":{}}}"#;

fn start(
    body: &'static str,
    diagnostics: Value,
) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
    let (address, _) = super::incremental::serving_registry(body)?;
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
fn an_outdated_range_gets_an_information_diagnostic_on_its_requirement() -> Result<()> {
    let (client, server_thread) = start(ASTRO, json!({}))?;
    let uri = "file:///workspace/astro/package.json";
    open(&client, uri, "astro", "^7.2.2")?;

    let diagnostics = settled_diagnostics(&client, uri)?;

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
    shutdown_client(&client, 2, server_thread)
}

#[test]
fn a_patch_only_floor_bump_keeps_its_lens_and_gets_no_diagnostic() -> Result<()> {
    let (client, server_thread) = start(
        r#"{"dist-tags":{"latest":"2.5.14"},"versions":{"2.5.8":{},"2.5.14":{}}}"#,
        json!({}),
    )?;
    let uri = "file:///workspace/biome/package.json";
    open(&client, uri, "@biomejs/biome", "^2.5.8")?;

    assert_eq!(settled_diagnostics(&client, uri)?, Vec::<Value>::new());
    let titles = super::incremental::lens_titles(&client, 2, uri)?;
    assert!(
        titles.iter().any(|title| title.contains("2.5.14")),
        "lenses: {titles:?}"
    );
    shutdown(&client, 3, server_thread)
}

#[test]
fn update_severity_off_publishes_no_update_diagnostics() -> Result<()> {
    let (client, server_thread) = start(ASTRO, json!({"updateSeverity": "off"}))?;
    let uri = "file:///workspace/off/package.json";
    open(&client, uri, "astro", "^7.2.2")?;

    assert_eq!(settled_diagnostics(&client, uri)?, Vec::<Value>::new());
    shutdown_client(&client, 2, server_thread)
}

#[test]
fn closing_a_document_clears_its_update_diagnostics() -> Result<()> {
    let (client, server_thread) = start(ASTRO, json!({"updateSeverity": "warn"}))?;
    let uri = "file:///workspace/close/package.json";
    open(&client, uri, "astro", "^7.2.2")?;
    let diagnostics = settled_diagnostics(&client, uri)?;
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["severity"], 2);

    send_notification(
        &client,
        "textDocument/didClose",
        json!({"textDocument": {"uri": uri}}),
    )?;
    loop {
        let Message::Notification(notification) = receive(&client)? else {
            continue;
        };
        if notification.method == "textDocument/publishDiagnostics"
            && notification.params["uri"] == uri
        {
            assert_eq!(notification.params["diagnostics"], json!([]));
            break;
        }
    }
    shutdown_client(&client, 2, server_thread)
}

/// Shuts down after skipping the publishes that a lens request can start.
fn shutdown(
    client: &Connection,
    id: i32,
    server_thread: thread::JoinHandle<Result<()>>,
) -> Result<()> {
    send_request(client, id, "shutdown", Value::Null)?;
    while !matches!(receive(client)?, Message::Response(response) if response.id == id.into()) {}
    send_notification(client, "exit", Value::Null)?;
    server_thread
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?
}

#[test]
fn code_actions_offer_each_bump_inline_without_registry_requests() -> Result<()> {
    let (address, requests) = super::incremental::serving_registry(
        r#"{"dist-tags":{"latest":"8.0.1"},"versions":{"7.2.2":{},"7.3.5":{},"8.0.1":{}}}"#,
    )?;
    let (client, server_thread) = start_configured_server(
        json!({"capabilities": {"workspace": {"workspaceEdit": {"documentChanges": true}}}}),
        json!({
            "suggestions": {"showVulnerabilities": false},
            "npm": {"apiUrl": format!("http://{address}")}
        }),
    )?;
    let uri = "file:///workspace/actions/package.json";
    open(&client, uri, "astro", "^7.2.2")?;
    let diagnostics = settled_diagnostics(&client, uri)?;
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
    )?;
    let actions = loop {
        if let Message::Response(response) = receive(&client)?
            && response.id == 2.into()
        {
            break ok_result(Message::Response(response))?;
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
    shutdown(&client, 3, server_thread)
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
    requests: super::incremental::Requests,
    uri: &'static str,
}

fn start(uri: &'static str) -> Result<Session> {
    start_with(uri, MANIFEST)
}

fn start_with(uri: &'static str, text: &str) -> Result<Session> {
    let (address, requests) = super::incremental::serving_registry(REGISTRY)?;
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
    fn execute(&self, id: i32, command: &str, arguments: Value) -> Result<()> {
        send_request(
            &self.client,
            id,
            "workspace/executeCommand",
            json!({"command": command, "arguments": [arguments]}),
        )
    }

    /// The next `workspace/applyEdit` request, skipping notifications.
    fn next_edit(&self) -> Result<lsp_server::Request> {
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

    fn answer_edit(&self, request: lsp_server::Request, result: Value) -> Result<()> {
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
                Message::Response(response) if response.id == id.into() => return Ok(response),
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

fn new_texts(request: &lsp_server::Request) -> Vec<String> {
    request.params["edit"]["documentChanges"][0]["edits"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|edit| edit["newText"].as_str().map(ToOwned::to_owned))
        .collect()
}

#[test]
fn bulk_update_applies_one_edit_for_every_dependency_at_the_level() -> Result<()> {
    let session = start("file:///workspace/bulk/package.json")?;
    session.execute(
        2,
        "versionlens.server.updateAll",
        json!({"uri": session.uri, "level": "minor"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(new_texts(&edit), ["1.1.0", "1.1.0"]);
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn update_applies_the_named_dependency_at_its_level() -> Result<()> {
    let session = start("file:///workspace/one/package.json")?;
    session.execute(
        2,
        "versionlens.server.update",
        json!({"uri": session.uri, "dependency": "alpha", "level": "major"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(new_texts(&edit), ["2.0.0"]);
    assert_eq!(
        edit.params["edit"]["documentChanges"][0]["edits"][0]["range"]["start"]["character"],
        41
    );
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn choose_build_applies_the_chosen_version() -> Result<()> {
    let session = start("file:///workspace/build/package.json")?;
    session.execute(
        2,
        "versionlens.server.chooseBuild",
        json!({"uri": session.uri, "dependency": "beta", "build": "1.1.0"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(new_texts(&edit), ["1.1.0"]);
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn sort_applies_the_dependencies_in_name_order() -> Result<()> {
    let session = start("file:///workspace/sort/package.json")?;
    session.execute(2, "versionlens.server.sort", json!({"uri": session.uri}))?;

    let edit = session.next_edit()?;
    let texts = new_texts(&edit).concat();
    assert!(
        texts.find("alpha") < texts.find("beta"),
        "sorted text: {texts}"
    );
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn a_rejected_edit_returns_the_editor_reason() -> Result<()> {
    let session = start("file:///workspace/rejected/package.json")?;
    session.execute(
        2,
        "versionlens.server.updateAll",
        json!({"uri": session.uri, "level": "latest"}),
    )?;

    let edit = session.next_edit()?;
    session.answer_edit(
        edit,
        json!({"applied": false, "failureReason": "file is read-only"}),
    )?;
    let error = session
        .response(2)?
        .response_result
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected an error"))?;
    assert_eq!(error.code, ErrorCode::RequestFailed as i32);
    assert!(
        error.message.contains("file is read-only"),
        "{}",
        error.message
    );
    session.shutdown(3)
}

#[test]
fn an_update_after_an_edit_applies_against_the_current_document() -> Result<()> {
    let session = start("file:///workspace/stale/package.json")?;
    send_notification(
        &session.client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": session.uri, "version": 2},
            "contentChanges": [{"text": r#"{"dependencies":{"beta":"1.1.0","alpha":"1.0.0"}}"#}]
        }),
    )?;
    session.execute(
        2,
        "versionlens.server.update",
        json!({"uri": session.uri, "dependency": "beta"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(
        edit.params["edit"]["documentChanges"][0]["textDocument"]["version"],
        2
    );
    assert_eq!(new_texts(&edit), ["2.0.0"]);
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn clear_cache_makes_the_next_resolution_fetch_again() -> Result<()> {
    let session = start("file:///workspace/cache/package.json")?;
    super::incremental::lens_titles(&session.client, 2, session.uri)?;
    let before = session.registry_requests();
    assert!(before > 0);

    session.execute(3, "versionlens.server.clearCache", json!({}))?;
    assert_eq!(session.response(3)?.response_result.ok(), Some(Value::Null));
    super::incremental::lens_titles(&session.client, 4, session.uri)?;

    assert!(
        session.registry_requests() > before,
        "no registry request after clearing the cache"
    );
    session.shutdown(5)
}

#[test]
fn manifest_info_describes_the_document() -> Result<()> {
    // One-line manifests cannot be sorted, so this one has a line per entry.
    let session = start_with(
        "file:///workspace/info/package.json",
        "{\n  \"dependencies\": {\n    \"beta\": \"1.0.0\",\n    \"alpha\": \"1.0.0\"\n  }\n}\n",
    )?;
    session.execute(
        2,
        "versionlens.server.manifestInfo",
        json!({"uri": session.uri}),
    )?;

    let info = session
        .response(2)?
        .response_result
        .ok()
        .ok_or_else(|| anyhow::anyhow!("expected a result"))?;
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
    session.shutdown(3)
}

#[test]
fn positional_arguments_are_rejected() -> Result<()> {
    let session = start("file:///workspace/positional/package.json")?;
    send_request(
        &session.client,
        2,
        "workspace/executeCommand",
        json!({"command": "versionlens.server.update", "arguments": [session.uri, "alpha"]}),
    )?;

    let error = session
        .response(2)?
        .response_result
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected an error"))?;
    assert_eq!(error.code, ErrorCode::InvalidParams as i32);
    session.shutdown(3)
}

/// The update command of the lens for `dependency`.
fn lens_command(session: &Session, id: i32, dependency: &str) -> Result<lsp_types::Command> {
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
fn a_lens_clicked_after_its_line_moved_updates_the_current_entry() -> Result<()> {
    let session = start("file:///workspace/moved/package.json")?;
    let command = lens_command(&session, 2, "beta")?;
    send_notification(
        &session.client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": session.uri, "version": 2},
            "contentChanges": [{"text": format!("\n{MANIFEST}")}]
        }),
    )?;
    send_request(
        &session.client,
        3,
        "workspace/executeCommand",
        serde_json::to_value(&command)?,
    )?;

    let edit = session.next_edit()?;
    let change = &edit.params["edit"]["documentChanges"][0];
    assert_eq!(change["textDocument"]["version"], 2);
    assert_eq!(change["edits"][0]["range"]["start"]["line"], 1);
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(3)?.response_result.ok(), Some(Value::Null));
    session.shutdown(4)
}

#[test]
fn an_update_with_a_version_and_no_level_applies_that_version() -> Result<()> {
    let session = start("file:///workspace/version/package.json")?;
    session.execute(
        2,
        "versionlens.server.update",
        json!({"uri": session.uri, "dependency": "alpha", "version": "1.1.0"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(new_texts(&edit), ["1.1.0"]);
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

fn workspace_root(name: &str) -> Result<(std::path::PathBuf, String)> {
    let root = versionlens_test_support::temporary_directory(name)?.canonicalize()?;
    std::fs::write(root.join(".gitignore"), "skipped/\n")?;
    for directory in ["first", "second", "skipped"] {
        std::fs::create_dir_all(root.join(directory))?;
        std::fs::write(root.join(directory).join("package.json"), MANIFEST)?;
    }
    let root_uri = versionlens_engine::workspace_file_uri(&root).context("missing root URI")?;
    Ok((root, root_uri))
}

fn start_workspace_update_server(root_uri: &str) -> Result<(Session, std::net::SocketAddr)> {
    let (address, requests) = super::incremental::serving_registry(REGISTRY)?;
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
fn changed_manifests(edit: &lsp_server::Request) -> Vec<(String, String)> {
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
fn workspace_update_edits_every_unignored_manifest_without_opening_it() -> Result<()> {
    let (root, root_uri) = workspace_root("versionlens-lsp-update-workspace")?;
    let (session, _) = start_workspace_update_server(&root_uri)?;
    session.execute(
        2,
        "versionlens.server.updateWorkspace",
        json!({"level": "minor"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(
        changed_manifests(&edit),
        [
            ("first".to_owned(), "1.1.0,1.1.0".to_owned()),
            ("second".to_owned(), "1.1.0,1.1.0".to_owned())
        ]
    );
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)?;
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn workspace_update_waits_for_pending_settings() -> Result<()> {
    let (root, root_uri) = workspace_root("versionlens-lsp-update-workspace-settings")?;
    let (session, address) = start_workspace_update_server(&root_uri)?;
    send_notification(
        &session.client,
        "workspace/didChangeConfiguration",
        json!({}),
    )?;
    session.execute(
        2,
        "versionlens.server.updateWorkspace",
        json!({"level": "minor"}),
    )?;
    answer_configuration(
        &session.client,
        json!({
            "suggestions": {"showVulnerabilities": false},
            "npm": {"apiUrl": format!("http://{address}")},
            "workspace": {"exclude": ["**/second/**"]}
        }),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(
        changed_manifests(&edit),
        [("first".to_owned(), "1.1.0,1.1.0".to_owned())]
    );
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)?;
    std::fs::remove_dir_all(root)?;
    Ok(())
}
}

mod authorization;

mod rate_limit;
