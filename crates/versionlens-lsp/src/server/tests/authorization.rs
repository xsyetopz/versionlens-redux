use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use lsp_server::{RequestId, Response};

use super::*;
use crate::server::run_connection_with;

const TOKEN: &str = "secret-token";
const RESPONSE_BUDGET: Duration = Duration::from_millis(500);

/// A registry that answers 401 until a request carries the token.
struct Registry {
    address: std::net::SocketAddr,
    /// The `Authorization` header of each request, in arrival order.
    requests: Arc<Mutex<Vec<Option<String>>>>,
    stop: Arc<AtomicBool>,
}

impl Registry {
    fn start() -> Result<Self> {
        Self::serve(false)
    }

    /// A registry that answers every request, with or without the token.
    fn open() -> Result<Self> {
        Self::serve(true)
    }

    fn serve(open: bool) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (log, flag) = (requests.clone(), stop.clone());
        // Detached: it stops on its own once `stop` is set.
        let _server = thread::spawn(move || {
            while !flag.load(Ordering::Acquire) {
                let Ok((socket, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                };
                let log = log.clone();
                thread::spawn(move || {
                    let _ = answer(socket, &log, open);
                });
            }
        });
        Ok(Self {
            address,
            requests,
            stop,
        })
    }

    fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    fn headers(&self) -> Vec<Option<String>> {
        self.requests
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn authorized(&self) -> usize {
        let expected = format!("Bearer {TOKEN}");
        self.headers()
            .iter()
            .filter(|header| header.as_deref() == Some(expected.as_str()))
            .count()
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

fn answer(
    mut socket: std::net::TcpStream,
    log: &Mutex<Vec<Option<String>>>,
    open: bool,
) -> Result<()> {
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    let mut buffer = [0; 8192];
    let read = socket.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..read]).into_owned();
    let header = request.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("authorization")
            .then(|| value.trim().to_owned())
    });
    log.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(header.clone());
    if open || header.as_deref() == Some(format!("Bearer {TOKEN}").as_str()) {
        let body = r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#;
        write!(
            socket,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )?;
    } else {
        write!(
            socket,
            "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )?;
    }
    Ok(())
}

/// What the client does with `versionlens/authorization`.
enum Reply {
    Token,
    MethodNotFound,
    /// Never answers.
    Silent,
}

fn reply_to(request: lsp_server::Request, reply: &Reply) -> Option<Response> {
    if request.method != "versionlens/authorization" {
        return Some(Response::new_ok(request.id, Value::Null));
    }
    match reply {
        Reply::Token => Some(Response::new_ok(
            request.id,
            json!({"scheme":"Bearer","token":TOKEN}),
        )),
        Reply::MethodNotFound => Some(Response::new_err(
            request.id,
            ErrorCode::MethodNotFound as i32,
            "method not found: versionlens/authorization".to_owned(),
        )),
        Reply::Silent => None,
    }
}

/// Answers the server's requests for `duration` and returns the
/// `versionlens/authorization` params it received.
fn serve(client: &Connection, duration: Duration, reply: &Reply) -> Result<Vec<Value>> {
    let deadline = Instant::now() + duration;
    let mut asked = Vec::new();
    while let Ok(message) = client.receiver.recv_deadline(deadline) {
        let Message::Request(request) = message else {
            continue;
        };
        if request.method == "versionlens/authorization" {
            asked.push(request.params.clone());
        }
        if let Some(response) = reply_to(request, reply) {
            client.sender.send(Message::Response(response))?;
        }
    }
    Ok(asked)
}

fn code_lens_titles(client: &Connection, id: i32, uri: &str, reply: &Reply) -> Result<String> {
    send_request(
        client,
        id,
        "textDocument/codeLens",
        json!({"textDocument":{"uri":uri}}),
    )?;
    let id = RequestId::from(id);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let message = client.receiver.recv_deadline(deadline)?;
        match message {
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
            Message::Request(request) => {
                if let Some(response) = reply_to(request, reply) {
                    client.sender.send(Message::Response(response))?;
                }
            }
            _ => {}
        }
    }
}

fn start(registry: &Registry) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
    start_with(registry, authorization_timeout())
}

fn authorization_timeout() -> Duration {
    Duration::from_secs(60)
}

fn start_with(
    registry: &Registry,
    timeout: Duration,
) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
    start_settings(
        json!({"suggestions":{"showVulnerabilities":false},"npm":{"apiUrl":registry.url()}}),
        timeout,
    )
}

fn start_settings(
    settings: Value,
    timeout: Duration,
) -> Result<(Connection, thread::JoinHandle<Result<()>>)> {
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection_with(&server, None, timeout));
    initialize_configured(
        &client,
        json!({"capabilities":{"workspace":{"codeLens":{"refreshSupport":true}}}}),
        settings,
    )?;
    Ok((client, server_thread))
}

#[test]
fn a_credential_from_the_client_is_sent_on_the_retry() -> Result<()> {
    let registry = Registry::start()?;
    let (client, server_thread) = start(&registry)?;
    let uri = "file:///workspace/package.json";
    open_json_document(
        &client,
        uri,
        1,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;

    let asked = serve(&client, Duration::from_secs(2), &Reply::Token)?;

    assert_eq!(asked, [json!({"host":"127.0.0.1","status":401})]);
    assert_eq!(registry.headers().first(), Some(&None));
    assert!(registry.authorized() >= 1, "{:?}", registry.headers());
    let titles = code_lens_titles(&client, 10, uri, &Reply::Token)?;
    assert!(titles.contains("2.0.0"), "{titles}");
    shut_down(&client, 11, server_thread)
}

#[test]
fn a_client_without_the_method_falls_back_and_is_not_asked_again() -> Result<()> {
    let registry = Registry::start()?;
    let (client, server_thread) = start(&registry)?;
    let first = "file:///workspace/a/package.json";
    let second = "file:///workspace/b/package.json";
    let other = "file:///workspace/c/jsr.json";
    open_json_document(
        &client,
        first,
        1,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;
    let mut asked = serve(&client, Duration::from_secs(1), &Reply::MethodNotFound)?;
    open_json_document(
        &client,
        second,
        1,
        "{\"dependencies\":{\"another\":\"1.0.0\"}}",
    )?;
    open_json_document(
        &client,
        other,
        1,
        "{\"name\":\"@scope/pkg\",\"version\":\"1.0.0\"}",
    )?;
    asked.extend(serve(
        &client,
        Duration::from_secs(1),
        &Reply::MethodNotFound,
    )?);

    assert_eq!(asked.len(), 1, "{asked:?}");
    assert_eq!(asked[0]["host"], "127.0.0.1");
    assert_eq!(registry.authorized(), 0);
    // Both documents reached the registry, unauthenticated, and none waited.
    assert!(registry.headers().len() >= 2, "{:?}", registry.headers());

    let started = Instant::now();
    code_lens_titles(&client, 10, other, &Reply::MethodNotFound)?;
    assert!(
        started.elapsed() < RESPONSE_BUDGET,
        "{:?}",
        started.elapsed()
    );
    shut_down(&client, 11, server_thread)
}

/// Shuts the server down, answering its requests and skipping notifications,
/// and checks that it does so within the budget.
fn shut_down(client: &Connection, id: i32, server: thread::JoinHandle<Result<()>>) -> Result<()> {
    send_request(client, id, "shutdown", Value::Null)?;
    let started = Instant::now();
    loop {
        match client.receiver.recv_timeout(RESPONSE_BUDGET)? {
            Message::Response(response) if response.id == RequestId::from(id) => break,
            Message::Request(request) => client
                .sender
                .send(Message::Response(Response::new_ok(request.id, Value::Null)))?,
            _ => {}
        }
    }
    assert!(started.elapsed() < RESPONSE_BUDGET);
    send_notification(client, "exit", Value::Null)?;
    server
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?
}

fn dependencies(count: usize, prefix: &str) -> String {
    let entries = (0..count)
        .map(|index| format!("\"{prefix}{index}\":\"1.0.0\""))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"dependencies\":{{{entries}}}}}")
}

#[test]
fn an_unanswered_question_does_not_block_other_hosts_or_shutdown() -> Result<()> {
    let stuck = Registry::start()?;
    let other = Registry::open()?;
    let (client, server_thread) = start_settings(
        json!({
            "suggestions":{"showVulnerabilities":false},
            "npm":{"apiUrl":stuck.url()},
            "cargo":{"apiUrl":other.url()},
        }),
        authorization_timeout(),
    )?;
    // More refused requests than there are registry permits.
    open_json_document(
        &client,
        "file:///workspace/stuck/package.json",
        1,
        &dependencies(24, "stuck"),
    )?;
    let asked = serve(&client, Duration::from_secs(1), &Reply::Silent)?;
    assert_eq!(asked.len(), 1, "{asked:?}");

    send_notification(
        &client,
        "textDocument/didOpen",
        json!({"textDocument": {
            "uri": "file:///workspace/other/Cargo.toml",
            "languageId": "toml",
            "version": 1,
            "text": "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1.0.0\"\n",
        }}),
    )?;
    serve(&client, Duration::from_secs(2), &Reply::Silent)?;
    assert!(!other.headers().is_empty(), "the other host was not asked");

    shut_down(&client, 20, server_thread)
}

#[test]
fn documents_on_one_host_share_one_question() -> Result<()> {
    let registry = Registry::start()?;
    let (client, server_thread) = start(&registry)?;
    for (index, directory) in ["a", "b", "c"].into_iter().enumerate() {
        open_json_document(
            &client,
            &format!("file:///workspace/{directory}/package.json"),
            1,
            &dependencies(4, &format!("dep{index}x")),
        )?;
    }
    let asked = serve(&client, Duration::from_secs(2), &Reply::Token)?;
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert!(registry.authorized() >= 12, "{:?}", registry.headers());
    shut_down(&client, 20, server_thread)
}

#[test]
fn an_answer_after_the_timeout_checks_the_dependency_again() -> Result<()> {
    let registry = Registry::start()?;
    let (client, server_thread) = start_with(&registry, Duration::from_millis(200))?;
    let uri = "file:///workspace/package.json";
    open_json_document(
        &client,
        uri,
        1,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;
    // Hold the question past the timeout, then answer it.
    let question = loop {
        match client.receiver.recv_timeout(Duration::from_secs(5))? {
            Message::Request(request) if request.method == "versionlens/authorization" => {
                break request;
            }
            Message::Request(request) => client
                .sender
                .send(Message::Response(Response::new_ok(request.id, Value::Null)))?,
            _ => {}
        }
    };
    serve(&client, Duration::from_millis(600), &Reply::Silent)?;
    assert_eq!(registry.authorized(), 0);
    client.sender.send(Message::Response(Response::new_ok(
        question.id,
        json!({"scheme":"Bearer","token":TOKEN}),
    )))?;
    serve(&client, Duration::from_secs(4), &Reply::Silent)?;
    assert!(registry.authorized() >= 1, "{:?}", registry.headers());
    let titles = code_lens_titles(&client, 10, uri, &Reply::Silent)?;
    assert!(titles.contains("2.0.0"), "{titles}");
    shut_down(&client, 20, server_thread)
}

#[test]
fn a_question_unanswered_at_the_timeout_is_not_asked_again() -> Result<()> {
    let registry = Registry::start()?;
    let (client, server_thread) = start_with(&registry, Duration::from_millis(200))?;
    open_json_document(
        &client,
        "file:///workspace/a/package.json",
        1,
        "{\"dependencies\":{\"example\":\"1.0.0\"}}",
    )?;
    let mut asked = serve(&client, Duration::from_secs(1), &Reply::Silent)?;
    open_json_document(
        &client,
        "file:///workspace/b/package.json",
        1,
        "{\"dependencies\":{\"another\":\"1.0.0\"}}",
    )?;
    asked.extend(serve(&client, Duration::from_secs(1), &Reply::Silent)?);
    assert_eq!(asked.len(), 1, "{asked:?}");
    shut_down(&client, 20, server_thread)
}
