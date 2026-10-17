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
            let _ = socket.set_read_timeout(Some(Duration::from_secs(3)));
            let mut buffer = [0; 4096];
            if socket.read(&mut buffer).unwrap_or(0) == 0 {
                continue;
            }
            count.fetch_add(1, Ordering::SeqCst);
            requested.store(true, Ordering::SeqCst);
            let body = r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#;
            let _ = write!(
                socket,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    Ok(address)
}

fn start_without_code_lens(
    address: std::net::SocketAddr,
) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
    start_workspace_server(json!({
        "capabilities": {},
        "initializationOptions": {
            "showVulnerabilities": false,
            "http": {"proxy": null, "timeoutMs": 1000},
            "providers": {"registryUrls": [{"ecosystem": "npm", "url": format!("http://{address}")}]}
        }
    }))
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
fn collect_until_idle(client: &Connection, uri: &str, requested: &AtomicBool) -> Result<Vec<Seen>> {
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
fn a_client_without_code_lens_support_gets_diagnostics_once_resolution_completes() -> Result<()> {
    let requested = Arc::new(AtomicBool::new(false));
    let address = npm_registry(Arc::clone(&requested), Arc::new(AtomicUsize::new(0)))?;
    let (client, server_thread) = start_without_code_lens(address)?;
    let uri = "file:///workspace/push/package.json";

    open_json_document(
        &client,
        uri,
        1,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;
    let seen = collect_until_idle(&client, uri, &requested)?;

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
    shutdown_client(&client, 2, server_thread)
}

#[test]
fn rapid_edits_are_resolved_once_after_the_debounce() -> Result<()> {
    let requested = Arc::new(AtomicBool::new(false));
    let address = npm_registry(Arc::clone(&requested), Arc::new(AtomicUsize::new(0)))?;
    let (client, server_thread) = start_without_code_lens(address)?;
    let uri = "file:///workspace/debounce/package.json";
    open_json_document(
        &client,
        uri,
        1,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;
    collect_until_idle(&client, uri, &requested)?;

    for (version, requirement) in [(2, "^1.0.0"), (3, "~1.0.0")] {
        send_notification(
            &client,
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": uri, "version": version},
                "contentChanges": [{"text": format!("{{\"dependencies\":{{\"example\":\"{requirement}\"}}}}")}]
            }),
        )?;
    }
    let early = collect_within(&client, uri, &requested, Duration::from_millis(200));
    assert!(
        early.is_empty(),
        "the edits were checked before the debounce ended: {} messages",
        early.len()
    );
    let seen = collect_until_idle(&client, uri, &requested)?;

    assert_eq!(statuses(&seen), ["busy", "idle"]);
    let versions = seen
        .iter()
        .filter_map(|seen| match seen {
            Seen::Diagnostics { version, .. } => Some(*version),
            Seen::Status(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(versions, [Some(3)]);
    shutdown_client(&client, 2, server_thread)
}

#[test]
fn an_edit_checked_early_by_a_workspace_reload_does_not_stay_busy() -> Result<()> {
    let requested = Arc::new(AtomicBool::new(false));
    let address = npm_registry(Arc::clone(&requested), Arc::new(AtomicUsize::new(0)))?;
    let (client, server_thread) = start_without_code_lens(address)?;
    let uri = "file:///workspace/reload/package.json";
    open_json_document(
        &client,
        uri,
        1,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;
    collect_until_idle(&client, uri, &requested)?;

    send_notification(
        &client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri, "version": 2},
            "contentChanges": [{"text": "{\"dependencies\":{\"example\":\"^1.0.0\"}}"}]
        }),
    )?;
    // A reload sends the unsaved text to checking before the debounce ends.
    send_notification(
        &client,
        "workspace/didChangeWorkspaceFolders",
        json!({"event": {"added": [], "removed": []}}),
    )?;
    let seen = collect_within(&client, uri, &requested, Duration::from_secs(2));

    assert_ne!(
        statuses(&seen).last(),
        Some(&"busy"),
        "the document stayed busy"
    );
    shutdown_client(&client, 2, server_thread)
}
