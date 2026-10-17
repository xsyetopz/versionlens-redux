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
        let (mut socket, _) = listener.accept()?;
        let mut buffer = [0; 4096];
        anyhow::ensure!(socket.read(&mut buffer)? > 0, "registry request was empty");
        started.send(())?;
        thread::sleep(Duration::from_secs(5));
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
    let lenses = await_response(&client, RequestId::from(2), RESPONSE_BUDGET)?;
    anyhow::ensure!(lenses.is_array(), "expected code lens array, got {lenses}");

    send_request(&client, 3, "shutdown", Value::Null)?;
    await_response(&client, RequestId::from(3), RESPONSE_BUDGET)?;
    send_notification(&client, "exit", Value::Null)?;
    server_thread
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?
}

fn await_response(client: &Connection, id: RequestId, budget: Duration) -> Result<Value> {
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
        if response_id != id {
            continue;
        }
        return response_result
            .map_err(|error| anyhow::anyhow!("expected success for {id}, got {error:?}"));
    }
}
