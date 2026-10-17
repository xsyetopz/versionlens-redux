use super::*;

#[test]
fn settings_are_pulled_for_the_folder_and_pulled_again_on_change() -> Result<()> {
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, None));
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
    let Message::Notification(error) = receive(&client)? else {
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
    // Both answers are invalid, so the configuration error shows which one
    // was applied: none for the older answer, one for the newest.
    let invalid = json!({"caching": {"duration": "invalid"}});
    answer(&requests[0], invalid.clone())?;
    assert!(
        matches!(
            receive_within(&client, Duration::from_millis(500)),
            Err(crossbeam_channel::RecvTimeoutError::Timeout)
        ),
        "the older answer was applied"
    );
    answer(&requests[1], invalid)?;
    let Message::Notification(error) = receive(&client)? else {
        bail!("expected the newest answer to be applied");
    };
    assert_eq!(error.method, "window/showMessage");
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

#[test]
fn each_folder_is_presented_with_its_own_settings() -> Result<()> {
    let (address, _) = super::incremental::serving_registry(
        r#"{"dist-tags":{"latest":"2.0.0","next":"3.0.0-beta.1"},"versions":{"1.0.0":{},"2.0.0":{},"3.0.0-beta.1":{}}}"#,
    )?;
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, None));
    let (with, without) = ("file:///workspace/with", "file:///workspace/without");
    initialize_workspace(
        &client,
        json!({
            "workspaceFolders": [
                {"uri": with, "name": "with"},
                {"uri": without, "name": "without"}
            ],
            "capabilities": {"workspace": {"configuration": true}}
        }),
    )?;
    let settings = |prereleases: bool| {
        json!({
            "suggestions": {"showVulnerabilities": false, "showPrereleasesOnStartup": prereleases},
            "npm": {"apiUrl": format!("http://{address}")}
        })
    };
    let Message::Request(request) = receive(&client)? else {
        bail!("expected the settings request");
    };
    assert_eq!(request.params["items"][0]["scopeUri"], with);
    assert_eq!(request.params["items"][2]["scopeUri"], without);
    client
        .sender
        .send(Message::Response(lsp_server::Response::new_ok(
            request.id,
            json!([settings(true), null, settings(false), null]),
        )))?;

    let manifest = r#"{"dependencies": {"alpha": "1.0.0"}}"#;
    let mut titles = Vec::new();
    for (id, folder) in [(10, with), (11, without)] {
        let uri = format!("{folder}/package.json");
        open_json_document(&client, &uri, 1, manifest)?;
        titles.push(super::incremental::lens_titles(&client, id, &uri)?);
    }
    let shows_prerelease =
        |titles: &[String]| titles.iter().any(|title| title.contains("3.0.0-beta.1"));
    assert!(
        shows_prerelease(&titles[0]),
        "with prereleases: {:?}",
        titles[0]
    );
    assert!(
        !shows_prerelease(&titles[1]),
        "without prereleases: {:?}",
        titles[1]
    );
    // Background checking may still publish, so skip to the shutdown answer.
    send_request(&client, 12, "shutdown", Value::Null)?;
    while !matches!(receive(&client)?, Message::Response(response) if response.id == 12.into()) {}
    send_notification(&client, "exit", Value::Null)?;
    server_thread
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?
}
