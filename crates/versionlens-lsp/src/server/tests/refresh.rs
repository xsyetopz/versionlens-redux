use std::io::{Read, Write};

use super::*;

#[test]
fn opening_a_manifest_checks_versions_and_requests_a_refresh() -> Result<()> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let registry = thread::spawn(move || -> Result<()> {
        let (mut socket, _) = listener.accept()?;
        socket.set_read_timeout(Some(Duration::from_secs(3)))?;
        let mut buffer = [0; 4096];
        anyhow::ensure!(socket.read(&mut buffer)? > 0, "empty registry request");
        let body = r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#;
        write!(
            socket,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )?;
        Ok(())
    });
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, false));
    initialize_workspace(
        &client,
        json!({
            "capabilities":{"workspace":{"codeLens":{"refreshSupport":true},"applyEdit":true,"workspaceEdit":{"documentChanges":true}}},
            "initializationOptions":{"showVulnerabilities":false,"http":{"proxy":null,"timeoutMs":1000},"providers":{"registryUrls":[{"ecosystem":"npm","url":format!("http://{address}")}]}}
        }),
    )?;
    let uri = "file:///workspace/package.json";
    open_json_document(
        &client,
        uri,
        3,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;
    assert_diagnostics(receive(&client)?, uri, true)?;
    let Message::Request(refresh) = receive(&client)? else {
        bail!("expected code lens refresh");
    };
    assert_eq!(refresh.method, "workspace/codeLens/refresh");
    client
        .sender
        .send(Message::Response(lsp_server::Response::new_ok(
            refresh.id,
            Value::Null,
        )))?;
    registry
        .join()
        .map_err(|_| anyhow::anyhow!("registry panicked"))??;

    let command = checked_command(&client, 2, uri, "2.0.0")?;
    send_request(
        &client,
        3,
        "workspace/executeCommand",
        serde_json::to_value(command)?,
    )?;
    let Message::Request(edit) = receive(&client)? else {
        bail!("expected update edit");
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
        )))?;
    assert_error(receive(&client)?, ErrorCode::RequestFailed)?;
    shutdown_client(&client, 4, server_thread)
}
