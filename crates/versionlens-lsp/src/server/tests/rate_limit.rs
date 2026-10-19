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
        let (count, flag) = (requests.clone(), stop.clone());
        // Detached: it stops on its own once `stop` is set.
        let _server = thread::spawn(move || {
            while !flag.load(Ordering::Acquire) {
                let Ok((mut socket, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                };
                count.fetch_add(1, Ordering::AcqRel);
                thread::spawn(move || {
                    let _ = socket.set_nonblocking(false);
                    let _ = socket.set_read_timeout(Some(Duration::from_secs(3)));
                    let _ = socket.read(&mut [0; 8192]);
                    let _ = write!(
                        socket,
                        "HTTP/1.1 403 Forbidden\r\nx-ratelimit-remaining: 0\r\n\
                         x-ratelimit-reset: {reset}\r\nContent-Length: 0\r\n\
                         Connection: close\r\n\r\n"
                    );
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

fn drain(client: &Connection, duration: Duration, seen: &mut Seen) -> Result<()> {
    let deadline = Instant::now() + duration;
    while let Ok(message) = client.receiver.recv_deadline(deadline) {
        match message {
            Message::Request(request) => {
                if request.method == "versionlens/authorization" {
                    seen.authorization_requests += 1;
                }
                client
                    .sender
                    .send(Message::Response(Response::new_ok(request.id, Value::Null)))?;
            }
            Message::Notification(notification) => match notification.method.as_str() {
                "window/showMessage" => {
                    let params: lsp_types::ShowMessageParams =
                        serde_json::from_value(notification.params)?;
                    assert_eq!(params.typ, lsp_types::MessageType::WARNING);
                    seen.warnings.push(params.message);
                }
                "textDocument/publishDiagnostics" => {
                    let params: PublishDiagnosticsParams =
                        serde_json::from_value(notification.params)?;
                    seen.error_diagnostics += params
                        .diagnostics
                        .iter()
                        .filter(|diagnostic| {
                            diagnostic.severity == Some(lsp_types::DiagnosticSeverity::ERROR)
                        })
                        .count();
                }
                _ => {}
            },
            Message::Response(_) => {}
        }
    }
    Ok(())
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
fn an_exhausted_rate_limit_warns_once_and_stops_requests_until_the_reset() -> Result<()> {
    let reset = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600;
    let registry = Exhausted::start(reset)?;
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
    )?;
    let uri = "file:///workspace/package.json";
    open_json_document(
        &client,
        uri,
        1,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;
    let mut seen = Seen::default();
    // Wait for the warning, which follows the first request, then let any
    // stray request show up before counting.
    let deadline = Instant::now() + Duration::from_secs(15);
    while seen.warnings.is_empty() && Instant::now() < deadline {
        drain(&client, Duration::from_millis(100), &mut seen)?;
    }
    drain(&client, Duration::from_millis(300), &mut seen)?;
    assert_eq!(registry.requests(), 1);

    // Another edit and another document, before the reset.
    send_notification(
        &client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri, "version": 2},
            "contentChanges": [{"text": "{\"dependencies\":{\"example\":\"1.1.0\"}}"}]
        }),
    )?;
    open_json_document(
        &client,
        "file:///workspace/other/package.json",
        1,
        "{\"dependencies\":{\"another\":\"1.0.0\"}}",
    )?;
    drain(&client, Duration::from_secs(2), &mut seen)?;

    assert_eq!(registry.requests(), 1, "a request was sent while held");
    assert_eq!(seen.warnings.len(), 1, "{:?}", seen.warnings);
    assert!(seen.warnings[0].contains("UTC"), "{:?}", seen.warnings);
    assert_eq!(seen.error_diagnostics, 0);
    assert_eq!(seen.authorization_requests, 0);
    let titles = lens_titles(&client, 10, uri)?;
    assert!(titles.contains("rate limited until"), "{titles}");

    send_request(&client, 11, "shutdown", Value::Null)?;
    drain(&client, Duration::from_millis(300), &mut seen)?;
    send_notification(&client, "exit", Value::Null)?;
    server_thread
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?
}
