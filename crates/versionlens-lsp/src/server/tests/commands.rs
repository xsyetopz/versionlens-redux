use super::*;

const REGISTRY: &str =
    r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"1.1.0":{},"2.0.0":{}}}"#;
const MANIFEST: &str = r#"{"dependencies":{"beta":"1.0.0","alpha":"1.0.0"}}"#;

struct Session {
    client: Connection,
    server: thread::JoinHandle<Result<()>>,
    requests: super::incremental::Requests,
    uri: &'static str,
}

fn start(uri: &'static str) -> Result<Session> {
    start_with(uri, MANIFEST)
}

fn start_with(uri: &'static str, text: &str) -> Result<Session> {
    let (address, requests) = super::incremental::serving_registry(REGISTRY)?;
    let (client, server) = start_configured_server(
        json!({"capabilities": {"workspace": {"applyEdit": true, "workspaceEdit": {"documentChanges": true}}}}),
        json!({
            "suggestions": {"showVulnerabilities": false},
            "npm": {"apiUrl": format!("http://{address}")}
        }),
    )?;
    open_json_document(&client, uri, 1, text)?;
    Ok(Session {
        client,
        server,
        requests,
        uri,
    })
}

impl Session {
    fn execute(&self, id: i32, command: &str, arguments: Value) -> Result<()> {
        send_request(
            &self.client,
            id,
            "workspace/executeCommand",
            json!({"command": command, "arguments": [arguments]}),
        )
    }

    /// The next `workspace/applyEdit` request, skipping notifications.
    fn next_edit(&self) -> Result<lsp_server::Request> {
        loop {
            match receive(&self.client)? {
                Message::Request(request) if request.method == "workspace/applyEdit" => {
                    return Ok(request);
                }
                Message::Notification(_) => {}
                message => bail!("expected a workspace edit, got {message:?}"),
            }
        }
    }

    fn answer_edit(&self, request: lsp_server::Request, result: Value) -> Result<()> {
        self.client
            .sender
            .send(Message::Response(lsp_server::Response::new_ok(
                request.id, result,
            )))?;
        Ok(())
    }

    /// The response to request `id`, skipping notifications.
    fn response(&self, id: i32) -> Result<lsp_server::Response> {
        loop {
            match receive(&self.client)? {
                Message::Response(response) if response.id == id.into() => return Ok(response),
                Message::Notification(_) => {}
                message => bail!("expected response {id}, got {message:?}"),
            }
        }
    }

    fn registry_requests(&self) -> usize {
        self.requests
            .lock()
            .map(|requests| requests.values().sum())
            .unwrap_or_default()
    }

    fn shutdown(self, id: i32) -> Result<()> {
        send_request(&self.client, id, "shutdown", Value::Null)?;
        self.response(id)?;
        send_notification(&self.client, "exit", Value::Null)?;
        self.server
            .join()
            .map_err(|_| anyhow::anyhow!("server panicked"))?
    }
}

fn new_texts(request: &lsp_server::Request) -> Vec<String> {
    request.params["edit"]["documentChanges"][0]["edits"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|edit| edit["newText"].as_str().map(ToOwned::to_owned))
        .collect()
}

#[test]
fn bulk_update_applies_one_edit_for_every_dependency_at_the_level() -> Result<()> {
    let session = start("file:///workspace/bulk/package.json")?;
    session.execute(
        2,
        "versionlens.server.updateAll",
        json!({"uri": session.uri, "level": "minor"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(new_texts(&edit), ["1.1.0", "1.1.0"]);
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn update_applies_the_named_dependency_at_its_level() -> Result<()> {
    let session = start("file:///workspace/one/package.json")?;
    session.execute(
        2,
        "versionlens.server.update",
        json!({"uri": session.uri, "dependency": "alpha", "level": "major"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(new_texts(&edit), ["2.0.0"]);
    assert_eq!(
        edit.params["edit"]["documentChanges"][0]["edits"][0]["range"]["start"]["character"],
        41
    );
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn choose_build_applies_the_chosen_version() -> Result<()> {
    let session = start("file:///workspace/build/package.json")?;
    session.execute(
        2,
        "versionlens.server.chooseBuild",
        json!({"uri": session.uri, "dependency": "beta", "build": "1.1.0"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(new_texts(&edit), ["1.1.0"]);
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn sort_applies_the_dependencies_in_name_order() -> Result<()> {
    let session = start("file:///workspace/sort/package.json")?;
    session.execute(2, "versionlens.server.sort", json!({"uri": session.uri}))?;

    let edit = session.next_edit()?;
    let texts = new_texts(&edit).concat();
    assert!(
        texts.find("alpha") < texts.find("beta"),
        "sorted text: {texts}"
    );
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn a_rejected_edit_returns_the_editor_reason() -> Result<()> {
    let session = start("file:///workspace/rejected/package.json")?;
    session.execute(
        2,
        "versionlens.server.updateAll",
        json!({"uri": session.uri, "level": "latest"}),
    )?;

    let edit = session.next_edit()?;
    session.answer_edit(
        edit,
        json!({"applied": false, "failureReason": "file is read-only"}),
    )?;
    let error = session
        .response(2)?
        .response_result
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected an error"))?;
    assert_eq!(error.code, ErrorCode::RequestFailed as i32);
    assert!(
        error.message.contains("file is read-only"),
        "{}",
        error.message
    );
    session.shutdown(3)
}

#[test]
fn an_update_after_an_edit_applies_against_the_current_document() -> Result<()> {
    let session = start("file:///workspace/stale/package.json")?;
    send_notification(
        &session.client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": session.uri, "version": 2},
            "contentChanges": [{"text": r#"{"dependencies":{"beta":"1.1.0","alpha":"1.0.0"}}"#}]
        }),
    )?;
    session.execute(
        2,
        "versionlens.server.update",
        json!({"uri": session.uri, "dependency": "beta"}),
    )?;

    let edit = session.next_edit()?;
    assert_eq!(
        edit.params["edit"]["documentChanges"][0]["textDocument"]["version"],
        2
    );
    assert_eq!(new_texts(&edit), ["2.0.0"]);
    session.answer_edit(edit, json!({"applied": true}))?;
    assert_eq!(session.response(2)?.response_result.ok(), Some(Value::Null));
    session.shutdown(3)
}

#[test]
fn clear_cache_makes_the_next_resolution_fetch_again() -> Result<()> {
    let session = start("file:///workspace/cache/package.json")?;
    super::incremental::lens_titles(&session.client, 2, session.uri)?;
    let before = session.registry_requests();
    assert!(before > 0);

    session.execute(3, "versionlens.server.clearCache", json!({}))?;
    assert_eq!(session.response(3)?.response_result.ok(), Some(Value::Null));
    super::incremental::lens_titles(&session.client, 4, session.uri)?;

    assert!(
        session.registry_requests() > before,
        "no registry request after clearing the cache"
    );
    session.shutdown(5)
}

#[test]
fn manifest_info_describes_the_document() -> Result<()> {
    // One-line manifests cannot be sorted, so this one has a line per entry.
    let session = start_with(
        "file:///workspace/info/package.json",
        "{\n  \"dependencies\": {\n    \"beta\": \"1.0.0\",\n    \"alpha\": \"1.0.0\"\n  }\n}\n",
    )?;
    session.execute(
        2,
        "versionlens.server.manifestInfo",
        json!({"uri": session.uri}),
    )?;

    let info = session
        .response(2)?
        .response_result
        .ok()
        .ok_or_else(|| anyhow::anyhow!("expected a result"))?;
    assert_eq!(info["supported"], true);
    assert_eq!(info["provider"], "npm");
    assert_eq!(info["canSortDependencies"], true);
    assert_eq!(info["installTaskConfigKey"], "npm.onSaveChanges");
    assert_eq!(info["dependencyCount"], 2);
    session.shutdown(3)
}

#[test]
fn positional_arguments_are_rejected() -> Result<()> {
    let session = start("file:///workspace/positional/package.json")?;
    send_request(
        &session.client,
        2,
        "workspace/executeCommand",
        json!({"command": "versionlens.server.update", "arguments": [session.uri, "alpha"]}),
    )?;

    let error = session
        .response(2)?
        .response_result
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected an error"))?;
    assert_eq!(error.code, ErrorCode::InvalidParams as i32);
    session.shutdown(3)
}
