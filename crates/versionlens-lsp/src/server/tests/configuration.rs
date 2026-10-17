use super::*;

#[test]
fn settings_are_pulled_for_the_folder_and_pulled_again_on_change() -> Result<()> {
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, false));
    let folder = "file:///workspace/project";
    initialize_workspace(
        &client,
        json!({
            "workspaceFolders": [{"uri": folder, "name": "project"}],
            "capabilities": {"workspace": {"configuration": true}}
        }),
    )?;
    let params = answer_configuration(&client, json!({}))?;
    assert_eq!(
        params,
        json!({"items": [
            {"scopeUri": folder, "section": "versionlens"},
            {"scopeUri": folder, "section": "files.exclude"}
        ]})
    );

    // The notification payload is ignored; the server asks again.
    send_notification(
        &client,
        "workspace/didChangeConfiguration",
        json!({"settings": {"versionlens": {"caching": {"duration": "invalid"}}}}),
    )?;
    let params = answer_configuration(&client, json!({"caching": {"duration": "invalid"}}))?;
    assert_eq!(params["items"][0]["section"], "versionlens");
    let Message::Notification(error) = client.receiver.recv_timeout(Duration::from_secs(10))?
    else {
        bail!("expected a configuration error message");
    };
    assert_eq!(error.method, "window/showMessage");

    shutdown_client(&client, 2, server_thread)
}

#[test]
fn a_client_without_configuration_support_gets_defaults_and_no_request() -> Result<()> {
    let (client, server_thread) = start_workspace_server(json!({"capabilities": {}}))?;
    send_notification(
        &client,
        "workspace/didChangeConfiguration",
        json!({"settings": {"versionlens": {"caching": {"duration": "invalid"}}}}),
    )?;
    assert!(matches!(
        client.receiver.recv_timeout(Duration::from_millis(500)),
        Err(crossbeam_channel::RecvTimeoutError::Timeout)
    ));
    shutdown_client(&client, 2, server_thread)
}

#[test]
fn an_answer_to_an_older_settings_request_is_dropped() -> Result<()> {
    let (client, server_thread) = start_configured_server(json!({}), json!({}))?;
    for _ in 0..2 {
        send_notification(&client, "workspace/didChangeConfiguration", json!({}))?;
    }
    let mut requests = Vec::new();
    while requests.len() < 2 {
        if let Message::Request(request) = receive(&client)?
            && request.method == "workspace/configuration"
        {
            requests.push(request);
        }
    }
    let answer = |request: &lsp_server::Request, settings: Value| {
        client
            .sender
            .send(Message::Response(lsp_server::Response::new_ok(
                request.id.clone(),
                json!([settings, null]),
            )))
    };
    // The newest answer is valid; the older one arrives late and is invalid,
    // so an error message means the stale answer was applied.
    answer(&requests[1], json!({}))?;
    answer(&requests[0], json!({"caching": {"duration": "invalid"}}))?;
    assert!(matches!(
        receive_within(&client, Duration::from_millis(500)),
        Err(crossbeam_channel::RecvTimeoutError::Timeout)
    ));
    shutdown_client(&client, 2, server_thread)
}

/// Receives the next message that is not a log or status notification.
fn receive_within(
    client: &Connection,
    timeout: Duration,
) -> Result<Message, crossbeam_channel::RecvTimeoutError> {
    loop {
        let message = client.receiver.recv_timeout(timeout)?;
        if !matches!(
            &message,
            Message::Notification(notification)
                if matches!(notification.method.as_str(), "window/logMessage" | "versionlens/status")
        ) {
            return Ok(message);
        }
    }
}
