use super::*;

#[test]
fn project_version_command_sends_a_versioned_edit_and_waits_for_the_editor() -> Result<()> {
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, false));
    initialize_client(&client)?;
    let uri = "file:///workspace/package.json";
    open_json_document(&client, uri, 7, "{\"version\":\"1.0.0\"}")?;
    assert_diagnostics(receive(&client)?, uri, true)?;
    let command = checked_command(&client, 20, uri, "1.0.1")?;
    send_request(
        &client,
        21,
        "workspace/executeCommand",
        serde_json::to_value(&command)?,
    )?;
    let Message::Request(request) = receive(&client)? else {
        bail!("expected workspace edit");
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
        )))?;
    assert_eq!(ok_result(receive(&client)?)?, Value::Null);

    send_notification(
        &client,
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":8},"contentChanges":[{"text":"{\"version\":\"2.0.0\"}"}]}),
    )?;
    assert_diagnostics(receive(&client)?, uri, true)?;
    send_request(
        &client,
        22,
        "workspace/executeCommand",
        serde_json::to_value(&command)?,
    )?;
    assert_error(receive(&client)?, ErrorCode::ContentModified)?;
    shutdown_client(&client, 23, server_thread)
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
        let (mut socket, _) = listener.accept()?;
        let mut buffer = [0; 4096];
        let received = socket.read(&mut buffer)?;
        anyhow::ensure!(received > 0, "registry request was empty");
        started.send(())?;
        wait_release.recv_timeout(Duration::from_secs(10))?;
        let body = r#"{"dist-tags":{"latest":"2.0.0"}}"#;
        let _ = write!(
            socket,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        Ok(())
    });
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, false));
    let mut config: versionlens_core::SessionConfig =
        versionlens_core::SessionConfigInput::default().into();
    config.show_vulnerabilities = false;
    config.http.proxy = None;
    config
        .providers
        .registry_urls
        .push(versionlens_core::RegistryUrlConfig {
            ecosystem: versionlens_model::Ecosystem::Npm,
            url: format!("http://{address}"),
        });
    send_request(
        &client,
        1,
        "initialize",
        json!({"capabilities":{},"initializationOptions":config}),
    )?;
    ok_result(receive(&client)?)?;
    send_notification(&client, "initialized", json!({}))?;
    let uri = "file:///workspace/package.json";
    open_json_document(
        &client,
        uri,
        1,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;
    receive(&client)?;
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
        assert_error(
            client.receiver.recv_timeout(Duration::from_secs(1))?,
            ErrorCode::RequestCanceled,
        )?;
    } else {
        send_notification(
            &client,
            "textDocument/didChange",
            json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":"{}"}]}),
        )?;
        assert_diagnostics(
            client.receiver.recv_timeout(Duration::from_secs(1))?,
            uri,
            true,
        )?;
        assert_error(
            client.receiver.recv_timeout(Duration::from_secs(1))?,
            ErrorCode::ContentModified,
        )?;
    }
    shutdown_client(&client, 3, server_thread)?;
    release.send(())?;
    registry
        .join()
        .map_err(|_| anyhow::anyhow!("registry panicked"))??;
    Ok(())
}
