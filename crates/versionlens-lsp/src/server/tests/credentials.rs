//! The stored URL authentication and its two code actions, against a local registry stub that
//! records the Authorization header each lookup sends.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;

use super::*;

const NPM: &str = r#"{"versions":{"1.0.0":{}},"dist-tags":{"latest":"1.0.0"}}"#;
const URI: &str = "file:///p/package.json";
const MANIFEST: &str = r#"{"dependencies":{"zod":"^1.0.0"}}"#;

/// Serves `NPM` on a local port and sends each request's Authorization header, or "", to the
/// returned receiver; also returns the registry's URL prefix.
fn registry() -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = crossbeam_channel::unbounded();
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
            let text = String::from_utf8_lossy(&request).to_lowercase();
            let header = text
                .lines()
                .find_map(|l| l.strip_prefix("authorization: "))
                .unwrap_or_default();
            drop(tx.send(header.trim().to_owned()));
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{NPM}",
                NPM.len()
            );
            drop(stream.write_all(response.as_bytes()));
        }
    });
    (format!("http://127.0.0.1:{port}/"), rx)
}

fn start(url: &str, stored: Option<PathBuf>, actions: bool) -> Client {
    let options = json!({ "vulnerabilities": false, "timeoutMs": 2000, "registries": { "npm": format!("{url}{{enc}}") } });
    let params = json!({
        "capabilities": { "window": { "showDocument": { "support": true } } },
        "initializationOptions": { "versionlens": options, "authenticationActions": actions },
    });
    Client::init_at(params, stored)
}

fn source_titles(client: &Client, id: i32) -> Vec<String> {
    let zero = json!({ "line": 0, "character": 0 });
    let params = json!({ "textDocument": { "uri": URI }, "range": { "start": zero, "end": zero }, "context": { "diagnostics": [], "only": ["source"] } });
    let actions = client.request(id, "textDocument/codeAction", params);
    actions
        .response_result
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["title"].as_str().unwrap().to_owned())
        .collect()
}

/// Runs `command` and answers the prompt it raises with the button titled `choice`, after
/// checking the prompt's message and buttons.
fn answer(client: &Client, id: i32, command: &str, message: &str, buttons: &[&str], choice: &str) {
    let params = json!({ "command": command, "arguments": [URI] });
    let request = Request::new(id.into(), "workspace/executeCommand".into(), params);
    client.conn.sender.send(request.into()).unwrap();
    let asked = client.next(|m| match m {
        Message::Request(r) if r.method == "window/showMessageRequest" => Some(r.clone()),
        _ => None,
    });
    assert_eq!(asked.params["message"], message);
    let titles: Vec<&str> = asked.params["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, buttons);
    let reply = Response::new_ok(asked.id, json!({ "title": choice }));
    client.conn.sender.send(reply.into()).unwrap();
}

fn shown(client: &Client, text: &str) {
    client.next(|m| match m {
        Message::Notification(n) if n.method == "window/showMessage" => n.params["message"]
            .as_str()
            .filter(|m| m.contains(text))
            .map(drop),
        _ => None,
    });
}

/// The next header the registry receives that `test` accepts, within 5 s.
fn sent(rx: &Receiver<String>, test: impl Fn(&str) -> bool) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        let header = rx
            .recv_timeout(left)
            .expect("no matching lookup reached the registry");
        if test(&header) {
            return header;
        }
    }
}

#[test]
fn adds_and_removes_url_authentication_through_prompts_and_stored_file() {
    let dir = std::env::temp_dir().join(format!("versionlens-auth-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    let path = dir.join("versionlens").join("authorization.json");
    let (url, headers) = registry();
    let client = start(&url, Some(path.clone()), true);
    let commands = &client.init["capabilities"]["executeCommandProvider"]["commands"];
    assert!(
        commands
            .as_array()
            .unwrap()
            .contains(&json!("versionlens.addAuthentication"))
    );
    client.open(URI, MANIFEST);
    assert_eq!(sent(&headers, |_| true), "");
    let titles = source_titles(&client, 2);
    for title in [
        "VersionLens: Add URL authentication",
        "VersionLens: Remove URL authentication",
    ] {
        assert!(
            titles.iter().any(|t| t == title),
            "{title} missing from {titles:?}"
        );
    }

    let kinds = crate::server::credentials::KINDS;
    let ask = format!("How does {url} authenticate?");
    answer(
        &client,
        3,
        "versionlens.addAuthentication",
        &ask,
        &kinds,
        kinds[1],
    );
    let opened = client.next(|m| match m {
        Message::Request(r) if r.method == "window/showDocument" => Some(r.params.clone()),
        _ => None,
    });
    assert_eq!(opened["uri"], workspace::file_uri(&path.to_string_lossy()));
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&text).unwrap(),
        json!({ &url: { "token": "" } })
    );
    let line = text.lines().nth(2).unwrap();
    assert_eq!(
        opened["selection"]["start"],
        json!({ "line": 2, "character": line.find("\"\"").unwrap() + 1 })
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    shown(
        &client,
        &format!("fill in the Bearer token for {url} and save"),
    );

    // The user fills the token in and saves; the next message makes the server reread the file.
    std::fs::write(&path, json!({ &url: { "token": "s3cret" } }).to_string()).unwrap();
    let later = std::time::SystemTime::now() + Duration::from_secs(2);
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(later)
        .unwrap();
    client.run(4, "versionlens.refresh", &json!([URI]));
    assert_eq!(sent(&headers, |h| !h.is_empty()), "bearer s3cret");

    answer(
        &client,
        5,
        "versionlens.removeAuthentication",
        "Forget the authentication of which URL?",
        &[&url],
        &url,
    );
    shown(&client, &format!("authentication removed for {url}"));
    assert_eq!(
        serde_json::from_str::<Value>(&std::fs::read_to_string(&path).unwrap()).unwrap(),
        json!({})
    );
    client.run(6, "versionlens.refresh", &json!([URI]));
    assert_eq!(sent(&headers, |_| true), "");
    // The message comes before the response, which `run` would wait for and skip it on the way.
    let params = json!({ "command": "versionlens.removeAuthentication", "arguments": [] });
    let request = Request::new(7.into(), "workspace/executeCommand".into(), params);
    client.conn.sender.send(request.into()).unwrap();
    shown(&client, "no stored url authentication");
    client.stop();
    drop(std::fs::remove_dir_all(&dir));
}

#[test]
fn keeps_authentication_actions_to_editors_that_ask() {
    let (url, _headers) = registry();
    let client = start(&url, None, false);
    let commands = &client.init["capabilities"]["executeCommandProvider"]["commands"];
    assert!(
        !commands
            .as_array()
            .unwrap()
            .contains(&json!("versionlens.addAuthentication"))
    );
    client.open(URI, MANIFEST);
    let titles = source_titles(&client, 2);
    assert!(
        !titles.iter().any(|t| t.contains("URL authentication")),
        "{titles:?}"
    );
    client.stop();
}
