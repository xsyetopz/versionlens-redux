use std::thread;

use super::*;

struct Client {
    conn: Connection,
    server: thread::JoinHandle<Result<(), String>>,
    init: Value,
}

impl Client {
    fn start() -> Self {
        Self::with(&json!({ "vulnerabilities": false, "timeoutMs": 500 }))
    }

    fn with(options: &Value) -> Self {
        Self::init(
            json!({ "capabilities": {}, "initializationOptions": { "versionlens": options } }),
        )
    }

    fn init(params: Value) -> Self {
        Self::init_at(params, None)
    }

    /// Starts a server that keeps URL authentication in `stored`; the others keep none, so no
    /// test reads the user's own file.
    fn init_at(params: Value, stored: Option<std::path::PathBuf>) -> Self {
        let (server, conn) = Connection::memory();
        let server = thread::spawn(move || serve(server, stored));
        let mut client = Self {
            conn,
            server,
            init: Value::Null,
        };
        client.init = client
            .request(1, "initialize", params)
            .response_result
            .unwrap();
        client.notify("initialized", json!({}));
        client
    }

    fn notify(&self, method: &str, params: Value) {
        self.conn
            .sender
            .send(Notification::new(method.into(), params).into())
            .unwrap();
    }

    /// Sends a request and returns its response, skipping the server's own notifications.
    fn request(&self, id: i32, method: &str, params: Value) -> Response {
        self.conn
            .sender
            .send(Request::new(id.into(), method.into(), params).into())
            .unwrap();
        loop {
            match self
                .conn
                .receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
            {
                Message::Response(response) if response.id == id.into() => return response,
                _ => {}
            }
        }
    }

    fn open(&self, uri: &str, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "", "version": 1, "text": text } }),
        );
    }

    fn stop(self) {
        self.request(99, "shutdown", Value::Null);
        self.notify("exit", Value::Null);
        self.server.join().unwrap().unwrap();
    }
}

#[test]
fn answers_lenses_and_hovers_from_memory_at_once() {
    let client = Client::start();
    client.open(
        "file:///p/libraries.hxml",
        "-lib heaps:1.10.0\n-lib format\n",
    );
    client.open("file:///p/requirements.txt", "requests==2.31.0\n");
    let started = Instant::now();
    let lenses = client.request(
        2,
        "textDocument/codeLens",
        json!({ "textDocument": { "uri": "file:///p/libraries.hxml" } }),
    );
    assert_eq!(lenses.response_result.unwrap().as_array().unwrap().len(), 2);
    let hover = client.request(3, "textDocument/hover", json!({ "textDocument": { "uri": "file:///p/requirements.txt" }, "position": { "line": 0, "character": 2 } }));
    assert!(
        hover
            .response_result
            .unwrap()
            .pointer("/contents/value")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("requests")
    );
    assert!(
        started.elapsed() < Duration::from_millis(500),
        "handlers waited on the network"
    );
    client.stop();
}

#[test]
fn reports_unknown_requests_and_commands() {
    let client = Client::start();
    let unknown = client.request(2, "textDocument/definition", json!({}));
    assert!(unknown.response_result.is_err());
    let command = client.request(
        3,
        "workspace/executeCommand",
        json!({ "command": "versionlens.nope", "arguments": [] }),
    );
    assert!(command.response_result.is_err());
    let toggle = client.request(
        4,
        "workspace/executeCommand",
        json!({ "command": "versionlens.toggle", "arguments": [] }),
    );
    assert!(toggle.response_result.is_ok());
    client.stop();
}

#[test]
fn advertises_globs_and_ignores_other_files() {
    let client = Client::start();
    let globs = client
        .init
        .pointer("/capabilities/experimental/fileTypes")
        .unwrap()
        .as_array()
        .unwrap();
    assert!(globs.iter().any(|g| g == "*.hxml"));
    let hover = client.request(2, "textDocument/hover", json!({ "textDocument": { "uri": "file:///p/main.ts" }, "position": { "line": 0, "character": 0 } }));
    assert_eq!(hover.response_result.unwrap(), Value::Null);
    client.stop();
}

#[test]
fn reads_package_shaped_json_and_only_files_on_disk() {
    let client = Client::start();
    let text = "{ \"dependencies\": { \"left-pad\": \"1.3.0\" } }\n";
    for uri in [
        "file:///p/web-module.json",
        "untitled:/p/package.json",
        "git:/p/package.json",
    ] {
        client.open(uri, text);
    }
    let lenses = |id, uri: &str| {
        client
            .request(
                id,
                "textDocument/codeLens",
                json!({ "textDocument": { "uri": uri } }),
            )
            .response_result
            .unwrap()
            .as_array()
            .map_or(0, Vec::len)
    };
    assert_eq!(lenses(2, "file:///p/web-module.json"), 1);
    assert_eq!(lenses(3, "untitled:/p/package.json"), 0);
    assert_eq!(lenses(4, "git:/p/package.json"), 0);
    client.stop();
}

#[test]
fn applies_incremental_changes() {
    let client = Client::start();
    let uri = "file:///p/requirements.txt";
    client.open(uri, "requests==2.31.0\n");
    let edit = json!({ "range": { "start": { "line": 1, "character": 0 }, "end": { "line": 1, "character": 0 } }, "text": "flask==2.0\n" });
    client.notify(
        "textDocument/didChange",
        json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [edit] }),
    );
    let lenses = client.request(
        2,
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 1, "character": 1 } }),
    );
    assert!(
        lenses
            .response_result
            .unwrap()
            .pointer("/contents/value")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("flask")
    );
    client.stop();
}

#[test]
fn decodes_file_uris() {
    assert_eq!(decode("/a%20b/%E2%82%AC.txt"), "/a b/€.txt");
}

#[test]
fn waits_for_exit_after_late_messages() {
    let client = Client::start();
    client.open("file:///p/requirements.txt", "requests==2.31.0\n");
    client.request(98, "shutdown", Value::Null);
    client.notify(
        "textDocument/didChange",
        json!({ "textDocument": { "uri": "file:///p/requirements.txt", "version": 2 }, "contentChanges": [{ "text": "flask\n" }] }),
    );
    let late = client.request(
        2,
        "textDocument/codeLens",
        json!({ "textDocument": { "uri": "file:///p/requirements.txt" } }),
    );
    assert_eq!(
        late.response_result.unwrap_err().code,
        ErrorCode::InvalidRequest as i32
    );
    client.notify("exit", Value::Null);
    client.server.join().unwrap().unwrap();
}

#[test]
fn updates_all_after_pending_lookups_and_names_failures() {
    // A blackhole address keeps the lookup in flight until the timeout fails it.
    let registries = json!({ "pypi": "http://10.255.255.1/{name}" });
    let client = Client::with(
        &json!({ "vulnerabilities": false, "timeoutMs": 500, "registries": registries }),
    );
    client.open("file:///p/requirements.txt", "requests==2.31.0\n");
    let args =
        json!({ "command": "versionlens.updateAll", "arguments": ["file:///p/requirements.txt"] });
    assert!(
        client
            .request(2, "workspace/executeCommand", args)
            .response_result
            .is_ok()
    );
    // Run before the lookup ends, the update would report the pending dependency as up to date.
    let shown = loop {
        match client
            .conn
            .receiver
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
        {
            Message::Notification(n) if n.method == "window/showMessage" => break n.params,
            _ => {}
        }
    };
    let message = shown["message"].as_str().unwrap();
    assert!(
        message.contains("1 dependencies could not be looked up"),
        "{message}"
    );
    client.stop();
}

mod commands;
mod credentials;
