//! Commands, status, and source actions against a local registry stub, so no test needs the
//! network.

use std::io::{Read, Write};
use std::net::TcpListener;

use super::*;

const NPM: &str = r#"{"versions":{"1.0.0":{},"1.0.5":{},"1.2.0":{},"2.0.0":{},"2.1.0-beta.1":{}},"dist-tags":{"latest":"2.0.0"}}"#;
const MANIFEST: &str =
    "{\n  \"dependencies\": {\n    \"zod\": \"^1.0.0\",\n    \"axios\": \"^2.0.0\"\n  }\n}\n";
const URI: &str = "file:///p/package.json";

/// Serves `NPM` for every request on a local port; returns the npm URL template.
fn registry() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut request = Vec::new();
            let mut buf = [0; 1024];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => request.extend_from_slice(&buf[..n]),
                }
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{NPM}",
                NPM.len()
            );
            drop(stream.write_all(response.as_bytes()));
        }
    });
    format!("http://127.0.0.1:{port}/{{enc}}")
}

fn options() -> Value {
    json!({ "vulnerabilities": false, "timeoutMs": 2000, "registries": { "npm": registry() } })
}

impl Client {
    /// The next server message that `pick` accepts, within 5 s.
    pub(super) fn next<T>(&self, mut pick: impl FnMut(&Message) -> Option<T>) -> T {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let message = self.conn.receiver.recv_timeout(left).unwrap();
            if let Some(found) = pick(&message) {
                return found;
            }
        }
    }

    fn status(&self, test: impl Fn(&Value) -> bool) -> Value {
        self.next(|m| match m {
            Message::Notification(n) if n.method == "versionlens/status" && test(&n.params) => {
                Some(n.params.clone())
            }
            _ => None,
        })
    }

    /// Runs a command and returns the edit it asks the editor to apply, which the server sends
    /// before or after its response.
    fn edit(&self, id: i32, command: &str, args: &Value) -> Value {
        let params = json!({ "command": command, "arguments": args });
        let request = Request::new(id.into(), "workspace/executeCommand".into(), params);
        self.conn.sender.send(request.into()).unwrap();
        let mut answered = false;
        let edit = self.next(|m| match m {
            Message::Request(r) if r.method == "workspace/applyEdit" => Some(r.params.clone()),
            Message::Response(r) if r.id == id.into() => {
                assert!(r.response_result.is_ok(), "{r:?}");
                answered = true;
                None
            }
            _ => None,
        });
        if !answered {
            self.next(|m| matches!(m, Message::Response(r) if r.id == id.into()).then_some(()));
        }
        edit
    }

    pub(super) fn run(&self, id: i32, command: &str, args: &Value) -> Response {
        self.request(
            id,
            "workspace/executeCommand",
            json!({ "command": command, "arguments": args }),
        )
    }

    /// Opens the manifest and returns its status once its lookups are done.
    fn ready(options: &Value) -> (Self, Value) {
        let client = Self::with(options);
        client.open(URI, MANIFEST);
        let status = client.status(|s| s["busy"] == false && s["dependencies"] == 2);
        (client, status)
    }
}

fn new_texts(edit: &Value, uri: &str) -> Vec<String> {
    edit["edit"]["changes"][uri]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["newText"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn reports_status_and_switches_versions_and_prereleases() {
    let (client, status) = Client::ready(&options());
    assert_eq!(status["outdated"], 1);
    assert_eq!(status["sortable"], true);
    assert!(
        client
            .run(2, "versionlens.hide", &json!([URI]))
            .response_result
            .is_ok()
    );
    client.status(|s| s["enabled"] == false);
    // An unrelated settings change keeps the versions hidden.
    let mut changed = options();
    changed["timeoutMs"] = json!(3000);
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "versionlens": changed } }),
    );
    let lenses = client.request(
        3,
        "textDocument/codeLens",
        json!({ "textDocument": { "uri": URI } }),
    );
    assert_eq!(lenses.response_result.unwrap(), json!([]));
    assert!(
        client
            .run(4, "versionlens.togglePrereleases", &json!([]))
            .response_result
            .is_ok()
    );
    client.status(|s| s["prereleases"] == true);
    client.stop();
}

#[test]
fn updates_all_by_level() {
    let (client, _) = Client::ready(&options());
    for (id, level, want) in [
        (2, "patch", "^1.0.5"),
        (3, "minor", "^1.2.0"),
        (4, "major", "^2.0.0"),
    ] {
        let edit = client.edit(id, "versionlens.updateAll", &json!([URI, level]));
        assert_eq!(new_texts(&edit, URI), [want], "{level}");
    }
    client.stop();
}

#[test]
fn sorts_dependencies() {
    let (client, _) = Client::ready(&options());
    let edit = client.edit(2, "versionlens.sort", &json!([URI]));
    assert_eq!(
        new_texts(&edit, URI),
        ["    \"axios\": \"^2.0.0\",\n    \"zod\": \"^1.0.0\""]
    );
    client.stop();
}

#[test]
fn offers_commands_as_source_actions() {
    let (client, _) = Client::ready(&options());
    let params = json!({ "textDocument": { "uri": URI }, "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } }, "context": { "diagnostics": [], "only": ["source"] } });
    let actions = client
        .request(2, "textDocument/codeAction", params)
        .response_result
        .unwrap();
    let titles: Vec<&str> = actions
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["title"].as_str().unwrap())
        .collect();
    for title in [
        "VersionLens: Update 1 dependency to latest",
        "VersionLens: Update workspace dependencies to latest",
        "VersionLens: Sort dependencies",
        "VersionLens: Hide versions",
        "VersionLens: Show prereleases",
    ] {
        assert!(titles.contains(&title), "{title} missing from {titles:?}");
    }
    assert!(
        !titles.iter().any(|t| t.starts_with("Update zod")),
        "{titles:?}"
    );
    client.stop();
}

#[test]
fn updates_every_manifest_in_workspace() {
    let root = std::env::temp_dir().join(format!("versionlens-ws-{}", std::process::id()));
    let nested = root.join("app");
    std::fs::create_dir_all(root.join("node_modules/x")).unwrap();
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(nested.join("package.json"), MANIFEST).unwrap();
    std::fs::write(root.join("node_modules/x/package.json"), MANIFEST).unwrap();
    let root_uri = workspace::file_uri(&root.to_string_lossy());
    let client = Client::init(json!({
        "capabilities": {},
        "rootUri": root_uri,
        "initializationOptions": { "versionlens": options() },
    }));
    let edit = client.edit(2, "versionlens.updateWorkspace", &json!(["major"]));
    let changes = edit["edit"]["changes"].as_object().unwrap();
    let manifest = workspace::file_uri(&nested.join("package.json").to_string_lossy());
    assert_eq!(changes.keys().collect::<Vec<_>>(), [&manifest]);
    assert_eq!(new_texts(&edit, &manifest), ["^2.0.0"]);
    client.stop();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn skips_workspace_manifests_exclude_glob_matches() {
    let root = std::env::temp_dir().join(format!("versionlens-wsx-{}", std::process::id()));
    for dir in ["app", "examples/demo"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
        std::fs::write(root.join(dir).join("package.json"), MANIFEST).unwrap();
    }
    let mut opts = options();
    opts["workspaceExclude"] = json!(["examples/**"]);
    let client = Client::init(json!({
        "capabilities": {},
        "rootUri": workspace::file_uri(&root.to_string_lossy()),
        "initializationOptions": { "versionlens": opts },
    }));
    let edit = client.edit(2, "versionlens.updateWorkspace", &json!(["major"]));
    let manifest = workspace::file_uri(&root.join("app/package.json").to_string_lossy());
    let changes = edit["edit"]["changes"].as_object().unwrap();
    assert_eq!(changes.keys().collect::<Vec<_>>(), [&manifest]);
    client.stop();
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn installs_when_save_changes_dependencies() {
    let dir = std::env::temp_dir().join(format!("versionlens-install-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let uri = workspace::file_uri(&dir.join("package.json").to_string_lossy());
    let mut opts = options();
    opts["install"] = json!({ "npm": "pwd" });
    let client = Client::with(&opts);
    client.open(&uri, MANIFEST);
    client.notify(
        "textDocument/didSave",
        json!({ "textDocument": { "uri": uri } }),
    );
    let change = json!({ "text": MANIFEST.replace("^1.0.0", "^1.2.0") });
    client.notify(
        "textDocument/didChange",
        json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [change] }),
    );
    client.notify(
        "textDocument/didSave",
        json!({ "textDocument": { "uri": uri } }),
    );
    let shown = client.next(|m| match m {
        Message::Notification(n) if n.method == "window/showMessage" => Some(n.params.clone()),
        _ => None,
    });
    assert_eq!(shown["message"], "VersionLens: pwd: done");
    client.stop();
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn heads_lenses_with_stats_and_uses_custom_indicators() {
    let mut opts = options();
    opts["stats"] = json!(true);
    opts["indicators"] = json!({ "update": "^^", "latest": "ok" });
    let (client, _) = Client::ready(&opts);
    let lenses = client.request(
        2,
        "textDocument/codeLens",
        json!({ "textDocument": { "uri": URI } }),
    );
    let lenses = lenses.response_result.unwrap();
    let titles: Vec<&str> = lenses
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["command"]["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles[0], "2 dependencies · ^^ 1 outdated");
    assert!(titles.contains(&"^^ latest 2.0.0"), "{titles:?}");
    assert!(titles.contains(&"ok latest 2.0.0"), "{titles:?}");
    assert!(titles.contains(&"🟡 satisfies 1.2.0"), "{titles:?}");
    assert!(titles.contains(&"^^ bump 1.2.0"), "{titles:?}");
    client.stop();
}

#[test]
fn reports_outdated_dependencies_at_configured_severity() {
    let mut opts = options();
    opts["updateSeverity"] = json!("hint");
    // Diagnostics go out before the status that `ready` waits for.
    let client = Client::with(&opts);
    client.open(URI, MANIFEST);
    let published = client.next(|m| match m {
        Message::Notification(n)
            if n.method == "textDocument/publishDiagnostics"
                && n.params["diagnostics"]
                    .as_array()
                    .is_some_and(|d| !d.is_empty()) =>
        {
            Some(n.params.clone())
        }
        _ => None,
    });
    let diagnostic = &published["diagnostics"][0];
    assert_eq!(diagnostic["severity"], 4);
    assert_eq!(diagnostic["message"], "zod can update to latest 2.0.0");
    client.stop();
}

#[test]
fn offers_only_prereleases_with_configured_tag() {
    let mut opts = options();
    opts["prereleases"] = json!(true);
    opts["prereleaseTags"] = json!(["rc"]);
    let (client, status) = Client::ready(&opts);
    // 2.1.0-beta.1 is left out, so axios ^2.0.0 stays current.
    assert_eq!(status["outdated"], 1);
    client.stop();
}

#[test]
fn reads_only_configured_file_types() {
    let mut opts = options();
    opts["fileTypes"] = json!(["cargo"]);
    let client = Client::with(&opts);
    client.open(URI, MANIFEST);
    let lenses = client.request(
        2,
        "textDocument/codeLens",
        json!({ "textDocument": { "uri": URI } }),
    );
    assert_eq!(lenses.response_result.unwrap(), Value::Null);
    client.stop();
}

#[test]
fn asks_for_build_and_applies_choice() {
    let client = Client::with(&options());
    let range =
        json!({ "start": { "line": 2, "character": 12 }, "end": { "line": 2, "character": 18 } });
    let args = json!([URI, range, "zod", ["1.0.0+b1", "1.0.0+b2"]]);
    let params = json!({ "command": "versionlens.build", "arguments": args });
    let request = Request::new(2.into(), "workspace/executeCommand".into(), params);
    client.conn.sender.send(request.into()).unwrap();
    let asked = client.next(|m| match m {
        Message::Request(r) if r.method == "window/showMessageRequest" => Some(r.clone()),
        _ => None,
    });
    assert_eq!(asked.params["message"], "Choose a build for zod");
    assert_eq!(asked.params["actions"][1]["title"], "1.0.0+b2");
    let choice = Response::new_ok(asked.id, json!({ "title": "1.0.0+b2" }));
    client.conn.sender.send(choice.into()).unwrap();
    let edit = client.next(|m| match m {
        Message::Request(r) if r.method == "workspace/applyEdit" => Some(r.params.clone()),
        _ => None,
    });
    assert_eq!(new_texts(&edit, URI), ["1.0.0+b2"]);
    client.stop();
}

#[test]
fn links_local_dependencies_to_their_directory() {
    let dir = std::env::temp_dir().join(format!("versionlens-dir-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("app")).unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    let uri = workspace::file_uri(&dir.join("app/package.json").to_string_lossy());
    let client = Client::with(&options());
    client.open(
        &uri,
        r#"{"dependencies":{"lib":"file:../lib","gone":"link:../gone"}}"#,
    );
    let lenses = client.request(
        2,
        "textDocument/codeLens",
        json!({ "textDocument": { "uri": uri } }),
    );
    let lenses = lenses.response_result.unwrap();
    let found = &lenses[0]["command"];
    assert_eq!(found["title"], "📁 file://../lib");
    let target = workspace::file_uri(&dir.join("app/../lib").to_string_lossy());
    assert_eq!(found["arguments"], json!([target]));
    assert_eq!(lenses[1]["command"]["title"], "🔴 not found ../gone");
    client.stop();
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn links_gems_in_path_block_to_its_directory() {
    // Arrange: a Gemfile whose bare gem sits in a `path` block.
    let dir = std::env::temp_dir().join(format!("versionlens-gem-dir-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("vendor/local")).unwrap();
    let uri = workspace::file_uri(&dir.join("Gemfile").to_string_lossy());
    let client = Client::with(&options());
    client.open(&uri, "path \"vendor/local\" do\n  gem \"local_one\"\nend\n");

    // Act
    let lenses = client.request(
        2,
        "textDocument/codeLens",
        json!({ "textDocument": { "uri": uri } }),
    );

    // Assert
    let lenses = lenses.response_result.unwrap();
    let target = workspace::file_uri(&dir.join("vendor/local").to_string_lossy());
    assert_eq!(lenses[0]["command"]["arguments"], json!([target]));
    client.stop();
    std::fs::remove_dir_all(dir).unwrap();
}
