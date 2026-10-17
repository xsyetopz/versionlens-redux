use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use super::*;

/// Background checking reruns after each edit, so the test waits this long
/// before it counts registry requests.
const SETTLE: Duration = Duration::from_millis(1500);

pub(super) type Requests = Arc<Mutex<BTreeMap<String, usize>>>;

#[test]
fn editing_a_manifest_resolves_only_changed_dependencies() -> Result<()> {
    let (address, requests) = counting_registry()?;
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, None));
    initialize_configured(
        &client,
        json!({"capabilities":{}}),
        json!({"suggestions":{"showVulnerabilities":false},"npm":{"apiUrl":format!("http://{address}")}}),
    )?;
    let uri = "file:///workspace/package.json";
    open_json_document(&client, uri, 1, &manifest("// first", "1.0.0"))?;
    lens_titles(&client, 10, uri)?;
    thread::sleep(SETTLE);
    let resolved = snapshot(&requests);
    anyhow::ensure!(
        resolved.contains_key("/alpha") && resolved.contains_key("/beta"),
        "expected both packages to be fetched, got {resolved:?}"
    );

    change_document(&client, uri, 2, &manifest("// second", "1.0.0"))?;
    let titles = lens_titles(&client, 11, uri)?;
    thread::sleep(SETTLE);
    assert_eq!(
        snapshot(&requests),
        resolved,
        "a comment edit made registry requests"
    );
    anyhow::ensure!(
        titles.iter().any(|title| title.contains("2.0.0")),
        "suggestions disappeared after a comment edit: {titles:?}"
    );

    change_document(&client, uri, 3, &manifest("// second", "2.0.0"))?;
    lens_titles(&client, 12, uri)?;
    thread::sleep(SETTLE);
    let after = snapshot(&requests);
    assert_eq!(
        after.get("/beta"),
        resolved.get("/beta"),
        "an alpha requirement edit fetched beta again"
    );
    send_request(&client, 13, "shutdown", Value::Null)?;
    loop {
        if let Message::Response(response) = receive(&client)? {
            anyhow::ensure!(response.id == 13.into(), "unexpected response {response:?}");
            break;
        }
    }
    send_notification(&client, "exit", Value::Null)?;
    server_thread
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?
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
        let lenses = serde_json::from_value::<Vec<CodeLens>>(
            response
                .response_result
                .map_err(|error| anyhow::anyhow!("code lens request failed: {error:?}"))?,
        )?;
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
    serving_registry(r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#)
}

/// Serves `body` for every package and counts requests by path.
pub(super) fn serving_registry(body: &'static str) -> Result<(std::net::SocketAddr, Requests)> {
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
            let _ = socket.set_read_timeout(Some(Duration::from_secs(3)));
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
            let _ = write!(
                socket,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    Ok((address, requests))
}
