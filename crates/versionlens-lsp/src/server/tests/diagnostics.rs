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
