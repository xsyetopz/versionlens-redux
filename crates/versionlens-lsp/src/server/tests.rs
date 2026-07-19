use std::thread;
use std::time::Duration;

use anyhow::{Result, bail};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request};
use lsp_types::{CodeLens, PublishDiagnosticsParams, ServerCapabilities};
use serde_json::{Value, json};

use super::run_connection;
use crate::state::DISPLAY_CODE_LENS_COMMAND;

#[test]
fn raw_loop_rejects_bad_requests_and_survives_bad_notifications() -> Result<()> {
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server));

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
            "capabilities": {}
        }),
    )?;
    let initialize_result = ok_result(receive(client)?)?;
    let capabilities =
        serde_json::from_value::<ServerCapabilities>(initialize_result["capabilities"].clone())?;
    assert!(capabilities.code_lens_provider.is_some());
    let execute_commands = capabilities
        .execute_command_provider
        .ok_or_else(|| anyhow::anyhow!("expected execute command capabilities"))?;
    assert_eq!(execute_commands.commands, [DISPLAY_CODE_LENS_COMMAND]);
    send_notification(client, "initialized", json!({}))
}

fn assert_visible_code_lenses(client: &Connection, uri: &str) -> Result<()> {
    send_request(
        client,
        5,
        "textDocument/codeLens",
        json!({"textDocument": {"uri": uri}}),
    )?;
    let lenses = serde_json::from_value::<Vec<CodeLens>>(ok_result(receive(client)?)?)?;
    assert!(!lenses.is_empty());
    assert!(lenses.iter().all(|lens| {
        lens.command.as_ref().is_some_and(|command| {
            !command.title.is_empty()
                && command.command == DISPLAY_CODE_LENS_COMMAND
                && command.arguments.is_none()
        })
    }));
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

fn send_notification(connection: &Connection, method: &str, params: Value) -> Result<()> {
    connection.sender.send(Message::Notification(Notification {
        method: method.to_owned(),
        params,
    }))?;
    Ok(())
}

fn receive(connection: &Connection) -> Result<Message> {
    Ok(connection.receiver.recv_timeout(Duration::from_secs(10))?)
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
