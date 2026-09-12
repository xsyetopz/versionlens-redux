use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request};
use lsp_types::{CodeLens, PublishDiagnosticsParams, ServerCapabilities};
use serde_json::{Value, json};

use super::run_connection;
use crate::state::DISPLAY_CODE_LENS_COMMAND;

#[test]
fn raw_loop_rejects_bad_requests_and_survives_bad_notifications() -> Result<()> {
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, false));

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

    let uri = "file:///workspace/project/package.json";
    send_notification(
        &client,
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "json",
                "version": 1,
                "text": "{\"version\":\"1.0.0\"}"
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
            "contentChanges": [{"text": "{\"version\":\"1.1.0\"}"}]
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
    let server_thread = thread::spawn(move || run_connection(&server, false));
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
    let root_uri = versionlens_core::workspace_file_uri(&root).context("missing root URI")?;
    let workflow_uri = versionlens_core::workspace_file_uri(&workflow.canonicalize()?)
        .context("missing workflow URI")?;
    let (client, server_thread) = start_workspace_server(json!({
        "rootUri": root_uri,
        "capabilities": {},
        "initializationOptions": {"showVulnerabilities": false}
    }))?;
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
fn clears_discovery_diagnostic_after_automatic_retry_recovers() -> Result<()> {
    let root = versionlens_test_support::temporary_directory("versionlens-lsp-retry")?;
    std::fs::remove_dir_all(&root)?;
    let root_uri = versionlens_core::workspace_file_uri(&root).context("missing root URI")?;
    let (client, server_thread) =
        start_workspace_server(json!({"rootUri": root_uri, "capabilities": {}}))?;

    assert_diagnostics(
        client.receiver.recv_timeout(Duration::from_secs(10))?,
        &root_uri,
        false,
    )?;
    assert!(matches!(
        client.receiver.recv_timeout(Duration::from_millis(500)),
        Err(crossbeam_channel::RecvTimeoutError::Timeout)
    ));
    std::fs::create_dir_all(&root)?;
    assert_diagnostics(
        client.receiver.recv_timeout(Duration::from_secs(10))?,
        &root_uri,
        true,
    )?;

    shutdown_client(&client, 2, server_thread)?;
    std::fs::remove_dir_all(root)?;
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
    let execute_commands = capabilities
        .execute_command_provider
        .ok_or_else(|| anyhow::anyhow!("expected execute command capabilities"))?;
    assert_eq!(
        execute_commands.commands,
        [
            DISPLAY_CODE_LENS_COMMAND,
            crate::state::UPDATE_DEPENDENCY_COMMAND
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

fn start_workspace_server(params: Value) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, false));
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
            Message::Notification(notification) if notification.method == "window/logMessage"
        ) || matches!(
            &message,
            Message::Notification(notification)
                if notification.method == "textDocument/publishDiagnostics"
                    && notification.params["diagnostics"].as_array().is_some_and(|diagnostics| {
                        !diagnostics.is_empty()
                            && diagnostics.iter().all(|diagnostic| {
                                diagnostic["source"] == "VersionLens workspace discovery"
                            })
                    })
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

fn assert_diagnostics(message: Message, expected_uri: &str, empty: bool) -> Result<()> {
    let Message::Notification(notification) = message else {
        bail!("expected diagnostics notification, got {message:?}");
    };
    assert_eq!(notification.method, "textDocument/publishDiagnostics");
    let params = serde_json::from_value::<PublishDiagnosticsParams>(notification.params)?;
    assert_eq!(params.uri.as_str(), expected_uri);
    assert_eq!(params.diagnostics.is_empty(), empty);
    Ok(())
}

mod updates;

mod refresh;
