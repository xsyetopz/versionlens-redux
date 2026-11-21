//! The language server: one loop over editor messages and finished lookups. Handlers answer from
//! memory; lookups run on a tokio runtime and report back over a channel, after which the server
//! asks the editor to refresh lenses and hints.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender, select};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::{MessageType, Uri};
use serde_json::{Value, json};

use crate::catalog::{FileType, file_type};
use crate::config::Settings;
use crate::extract::{Dep, extract};
use crate::http::Http;
use crate::osv::{self, Query};
use crate::registry::{self, Registry};
use crate::render;
use crate::version::{Req, Version};

const COMMANDS: &[&str] = &[
    "versionlens.update",
    "versionlens.updateAll",
    "versionlens.refresh",
    "versionlens.clearCache",
    "versionlens.toggle",
    "versionlens.open",
];
/// How long a failed lookup is remembered before the next edit retries it.
const RETRY: Duration = Duration::from_secs(60);
/// Lookups finishing within this window share one refresh.
const COALESCE: Duration = Duration::from_millis(120);

type Key = (&'static str, String, Option<String>);

struct Found {
    versions: Vec<Version>,
    tagged: Option<Version>,
}

struct Entry {
    at: Instant,
    result: Result<Arc<Found>, String>,
}

struct Doc {
    uri: Uri,
    path: String,
    text: String,
    file: &'static FileType,
    deps: Vec<Dep>,
}

enum Event {
    Fetched(Key, Result<registry::Versions, String>, Duration),
    Osv(Vec<Query>, Result<Vec<Vec<String>>, String>),
}

enum Pending {
    Config,
    Progress(String),
    Ignore,
}

#[derive(Default)]
struct Caps {
    lens_refresh: bool,
    hint_refresh: bool,
    progress: bool,
    configuration: bool,
    show_document: bool,
}

struct Server {
    conn: Connection,
    runtime: tokio::runtime::Runtime,
    http: Http,
    settings: Settings,
    caps: Caps,
    docs: HashMap<String, Doc>,
    cache: HashMap<Key, Entry>,
    inflight: HashSet<Key>,
    /// Manifests whose `updateAll` waits for their lookups to finish.
    update_after: HashSet<String>,
    vulns: HashMap<Query, Vec<String>>,
    osv_inflight: HashSet<Query>,
    osv_warned: bool,
    events: (Sender<Event>, Receiver<Event>),
    pending: HashMap<RequestId, Pending>,
    next_id: i32,
    dirty: Option<Instant>,
    progress: Option<(String, bool, usize)>,
}

fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn key(dep: &Dep) -> Key {
    (dep.registry, dep.name.clone(), dep.source.clone())
}

fn param<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|e| format!("invalid params: {e}"))
}

/// Runs the server over stdio until the editor exits.
///
/// # Errors
/// Reports a protocol or transport failure.
pub(crate) fn run() -> Result<(), String> {
    let (conn, io) = Connection::stdio();
    serve(conn)?;
    io.join().map_err(|e| e.to_string())
}

fn serve(conn: Connection) -> Result<(), String> {
    let (id, params) = conn.initialize_start().map_err(|e| e.to_string())?;
    let capabilities = json!({
        "textDocumentSync": { "openClose": true, "change": 2 },
        "codeLensProvider": { "resolveProvider": false },
        "inlayHintProvider": true,
        "hoverProvider": true,
        "codeActionProvider": { "codeActionKinds": [render::ACTION_KIND] },
        "executeCommandProvider": { "commands": COMMANDS },
        "experimental": { "fileTypes": crate::catalog::globs() },
    });
    let info = json!({ "name": "versionlens", "version": env!("CARGO_PKG_VERSION") });
    conn.initialize_finish(
        id,
        json!({ "capabilities": capabilities, "serverInfo": info }),
    )
    .map_err(|e| e.to_string())?;
    let flag = |path: &str| {
        params
            .pointer(path)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    };
    let caps = Caps {
        lens_refresh: flag("/capabilities/workspace/codeLens/refreshSupport"),
        hint_refresh: flag("/capabilities/workspace/inlayHint/refreshSupport"),
        progress: flag("/capabilities/window/workDoneProgress"),
        configuration: flag("/capabilities/workspace/configuration"),
        show_document: flag("/capabilities/window/showDocument/support"),
    };
    let settings = params
        .get("initializationOptions")
        .map_or_default(Settings::from_json);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let http = Http::new(settings.clone())?;
    let mut server = Server {
        conn,
        runtime,
        http,
        settings,
        caps,
        docs: HashMap::new(),
        cache: HashMap::new(),
        inflight: HashSet::new(),
        update_after: HashSet::new(),
        vulns: HashMap::new(),
        osv_inflight: HashSet::new(),
        osv_warned: false,
        events: crossbeam_channel::unbounded(),
        pending: HashMap::new(),
        next_id: 0,
        dirty: None,
        progress: None,
    };
    server.log(
        MessageType::INFO,
        format!(
            "versionlens {} ready: {} file types",
            env!("CARGO_PKG_VERSION"),
            crate::catalog::FILE_TYPES.len()
        ),
    );
    server.run();
    Ok(())
}

impl Server {
    fn run(&mut self) {
        let messages = self.conn.receiver.clone();
        let events = self.events.1.clone();
        loop {
            let wait = self.dirty.map_or(Duration::from_secs(3600), |at| {
                (at + COALESCE).saturating_duration_since(Instant::now())
            });
            select! {
                recv(messages) -> message => match message {
                    Ok(message) => if self.message(message) { return },
                    Err(_) => return,
                },
                recv(events) -> event => if let Ok(event) = event { self.event(event) },
                default(wait) => self.flush(),
            }
        }
    }

    fn send(&self, message: impl Into<Message>) {
        // A closed channel means the editor is gone; the receive side ends the loop.
        drop(self.conn.sender.send(message.into()));
    }

    /// Waits for `exit` after `shutdown`. Editors may still deliver queued notifications, which
    /// are dropped, and requests, which the protocol answers with `InvalidRequest`. Returning
    /// before `exit` arrives would make the stdio reader fail to forward it.
    fn await_exit(&self) {
        for message in &self.conn.receiver {
            match message {
                Message::Notification(n) if n.method == "exit" => return,
                Message::Request(r) => self.send(Response::new_err(
                    r.id,
                    ErrorCode::InvalidRequest as i32,
                    "the server is shutting down".to_owned(),
                )),
                Message::Notification(_) | Message::Response(_) => {}
            }
        }
    }

    fn notify(&self, method: &str, params: Value) {
        self.send(Notification::new(method.to_owned(), params));
    }

    fn request(&mut self, method: &str, params: Value, pending: Pending) {
        self.next_id += 1;
        let id = RequestId::from(self.next_id);
        self.pending.insert(id.clone(), pending);
        self.send(Request::new(id, method.to_owned(), params));
    }

    fn log(&self, kind: MessageType, text: impl Into<String>) {
        let text: String = text.into();
        self.notify(
            "window/logMessage",
            json!({ "type": kind, "message": text }),
        );
    }

    fn message(&mut self, message: Message) -> bool {
        match message {
            Message::Request(request) => {
                if request.method == "shutdown" {
                    self.send(Response::new_ok(request.id, ()));
                    self.await_exit();
                    return true;
                }
                let (id, method) = (request.id.clone(), request.method.clone());
                let response = match self.handle(&method, request.params) {
                    Ok(result) => Response::new_ok(id, result),
                    Err((code, text)) => {
                        self.log(MessageType::ERROR, format!("{method}: {text}"));
                        Response::new_err(id, code as i32, text)
                    }
                };
                self.send(response);
            }
            Message::Response(response) => {
                match (self.pending.remove(&response.id), response.response_result) {
                    (Some(Pending::Config), Ok(result)) => {
                        self.configure(result.get(0).unwrap_or(&Value::Null));
                    }
                    (Some(Pending::Progress(token)), Ok(_)) => self.begin_progress(&token),
                    (_, Err(error)) => self.log(
                        MessageType::WARNING,
                        format!("editor rejected a request: {}", error.message),
                    ),
                    (_, Ok(_)) => {}
                }
            }
            Message::Notification(note) => {
                if let Err(text) = self.notification(&note.method, note.params) {
                    self.log(MessageType::ERROR, format!("{}: {text}", note.method));
                }
            }
        }
        false
    }

    fn notification(&mut self, method: &str, params: Value) -> Result<(), String> {
        match method {
            "initialized" => self.pull_config(),
            "workspace/didChangeConfiguration" => match params
                .get("settings")
                .filter(|s| s.get("versionlens").is_some())
            {
                Some(settings) => self.configure(settings),
                None => self.pull_config(),
            },
            "textDocument/didOpen" => {
                let p: lsp_types::DidOpenTextDocumentParams = param(params)?;
                self.open(&p.text_document.uri, p.text_document.text);
            }
            "textDocument/didChange" => {
                let p: lsp_types::DidChangeTextDocumentParams = param(params)?;
                let uri = p.text_document.uri;
                let Some(mut text) = self.docs.get(uri.as_str()).map(|d| d.text.clone()) else {
                    return Ok(());
                };
                for change in p.content_changes {
                    match change.range {
                        Some(range) => {
                            let lines = render::Lines::new(&text);
                            let (a, b) = (lines.offset(range.start), lines.offset(range.end));
                            text.replace_range(a.min(b)..a.max(b), &change.text);
                        }
                        None => text = change.text,
                    }
                }
                self.open(&uri, text);
            }
            "textDocument/didClose" => {
                let p: lsp_types::DidCloseTextDocumentParams = param(params)?;
                if self.docs.remove(p.text_document.uri.as_str()).is_some() {
                    self.notify(
                        "textDocument/publishDiagnostics",
                        json!({ "uri": p.text_document.uri, "diagnostics": [] }),
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn pull_config(&mut self) {
        if self.caps.configuration {
            self.request(
                "workspace/configuration",
                json!({ "items": [{ "section": "versionlens" }] }),
                Pending::Config,
            );
        }
    }

    fn configure(&mut self, value: &Value) {
        let settings = Settings::from_json(value);
        if settings == self.settings {
            return;
        }
        let refetch = settings.registries != self.settings.registries
            || settings.github_token != self.settings.github_token;
        match Http::new(settings.clone()) {
            Ok(http) => self.http = http,
            Err(error) => self.log(
                MessageType::ERROR,
                format!("could not apply HTTP settings: {error}"),
            ),
        }
        self.settings = settings;
        let s = &self.settings;
        let text = format!(
            "settings applied: enabled {}, prereleases {}, vulnerabilities {}, timeout {:?}, concurrency {}, registry overrides {:?}",
            s.enabled,
            s.prereleases,
            s.vulnerabilities,
            s.timeout,
            s.concurrency,
            s.registries.keys().collect::<Vec<_>>()
        );
        self.log(MessageType::INFO, text);
        if refetch {
            self.cache.clear();
        }
        self.schedule_all();
        self.dirty = Some(Instant::now());
    }

    fn open(&mut self, uri: &Uri, text: String) {
        let path = decode(uri.as_str().trim_start_matches("file://"));
        let Some(file) = file_type(&path) else {
            self.log(
                MessageType::LOG,
                format!("{path}: not a supported manifest; skipped"),
            );
            return;
        };
        let started = Instant::now();
        let deps = extract(file, &text);
        let first = !self.docs.contains_key(uri.as_str());
        if first {
            self.log(
                MessageType::INFO,
                format!(
                    "{path}: {} manifest, {} dependencies, read in {:?}",
                    file.id,
                    deps.len(),
                    started.elapsed()
                ),
            );
        }
        self.docs.insert(
            uri.as_str().to_owned(),
            Doc {
                uri: uri.clone(),
                path,
                text,
                file,
                deps,
            },
        );
        self.schedule(uri.as_str());
        self.dirty = Some(Instant::now());
    }

    fn schedule_all(&mut self) {
        let uris: Vec<String> = self.docs.keys().cloned().collect();
        for uri in uris {
            self.schedule(&uri);
        }
    }

    fn schedule(&mut self, uri: &str) {
        let Some(doc) = self.docs.get(uri) else {
            return;
        };
        let ttl = self.settings.cache;
        let wanted: Vec<Key> = doc.deps.iter().map(key).collect();
        for key in wanted {
            let fresh = self
                .cache
                .get(&key)
                .is_some_and(|e| e.at.elapsed() < if e.result.is_ok() { ttl } else { RETRY });
            let Some(registry) = registry::get(key.0) else {
                continue;
            };
            if fresh || !self.inflight.insert(key.clone()) {
                continue;
            }
            self.fetch(registry, key);
        }
        self.start_progress();
        self.check_vulns(uri);
    }

    fn fetch(&self, registry: &'static Registry, key: Key) {
        let (http, tx) = (self.http.clone(), self.events.0.clone());
        self.runtime.spawn(async move {
            let started = Instant::now();
            let result = http.versions(registry, &key.1, key.2.as_deref()).await;
            drop(tx.send(Event::Fetched(key, result, started.elapsed())));
        });
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::Fetched(key, result, took) => {
                self.inflight.remove(&key);
                let result = match result {
                    Ok(found) => {
                        self.log(
                            MessageType::LOG,
                            format!(
                                "{} {}: {} versions in {took:?}",
                                key.0,
                                key.1,
                                found.list.len()
                            ),
                        );
                        let mut versions: Vec<Version> = found
                            .list
                            .iter()
                            .filter_map(|v| Version::parse(v))
                            .collect();
                        versions.sort();
                        Ok(Arc::new(Found {
                            versions,
                            tagged: found.latest.as_deref().and_then(Version::parse),
                        }))
                    }
                    Err(error) => {
                        self.log(
                            MessageType::WARNING,
                            format!("{} {}: {error}", key.0, key.1),
                        );
                        Err(error)
                    }
                };
                self.cache.insert(
                    key,
                    Entry {
                        at: Instant::now(),
                        result,
                    },
                );
                let uris: Vec<String> = self.docs.keys().cloned().collect();
                for uri in uris {
                    self.check_vulns(&uri);
                }
            }
            Event::Osv(queries, result) => {
                for query in &queries {
                    self.osv_inflight.remove(query);
                }
                match result {
                    Ok(ids) => self.vulns.extend(queries.into_iter().zip(ids)),
                    Err(error) => {
                        self.log(
                            MessageType::ERROR,
                            format!("vulnerability check failed: {error}"),
                        );
                        if !self.osv_warned {
                            self.osv_warned = true;
                            let text = format!(
                                "VersionLens could not check vulnerabilities: {error}. Details are in the VersionLens output."
                            );
                            self.notify(
                                "window/showMessage",
                                json!({ "type": MessageType::WARNING, "message": text }),
                            );
                        }
                    }
                }
            }
        }
        self.dirty.get_or_insert_with(Instant::now);
    }

    /// Asks OSV about a document once every lookup it needs has finished.
    fn check_vulns(&mut self, uri: &str) {
        let Some(doc) = self.docs.get(uri) else {
            return;
        };
        if !self.settings.vulnerabilities
            || doc.deps.iter().any(|d| self.inflight.contains(&key(d)))
        {
            return;
        }
        let queries: Vec<Query> = doc
            .deps
            .iter()
            .filter_map(|dep| self.query(dep))
            .filter(|q| !self.vulns.contains_key(q) && !self.osv_inflight.contains(q))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        if queries.is_empty() {
            return;
        }
        self.osv_inflight.extend(queries.iter().cloned());
        let (http, tx) = (self.http.clone(), self.events.0.clone());
        self.runtime.spawn(async move {
            let result = osv::query(&http, &queries).await;
            drop(tx.send(Event::Osv(queries, result)));
        });
    }

    fn query(&self, dep: &Dep) -> Option<Query> {
        let ecosystem = registry::get(dep.registry)?.osv?;
        let found = self.cache.get(&key(dep))?.result.as_ref().ok()?;
        let req = Req::parse(&dep.req, registry::get(dep.registry)?.bare)?;
        let version = render::current(&req, &found.versions)?;
        Some(Query {
            ecosystem,
            name: dep.name.clone(),
            version: version.raw,
        })
    }

    fn flush(&mut self) {
        self.dirty = None;
        for doc in self.docs.values() {
            let diagnostics = if self.settings.enabled {
                render::diagnostics(&self.views(doc))
            } else {
                Vec::new()
            };
            self.notify(
                "textDocument/publishDiagnostics",
                json!({ "uri": doc.uri, "diagnostics": diagnostics }),
            );
        }
        if self.caps.lens_refresh {
            self.request("workspace/codeLens/refresh", Value::Null, Pending::Ignore);
        }
        if self.caps.hint_refresh {
            self.request("workspace/inlayHint/refresh", Value::Null, Pending::Ignore);
        }
        self.report_progress();
        self.run_deferred_updates();
    }

    fn start_progress(&mut self) {
        if self.caps.progress && self.progress.is_none() && !self.inflight.is_empty() {
            let token = format!("versionlens/{}", self.next_id + 1);
            self.progress = Some((token.clone(), false, self.inflight.len()));
            self.request(
                "window/workDoneProgress/create",
                json!({ "token": token }),
                Pending::Progress(token),
            );
        } else if let Some((_, _, total)) = &mut self.progress {
            *total = (*total).max(self.inflight.len());
        }
    }

    fn begin_progress(&mut self, token: &str) {
        if let Some((current, started, total)) = &mut self.progress
            && current == token
        {
            *started = true;
            let value = json!({ "kind": "begin", "title": "VersionLens", "message": format!("checking {total} packages"), "percentage": 0 });
            self.notify("$/progress", json!({ "token": token, "value": value }));
        }
    }

    fn report_progress(&mut self) {
        let Some((token, started, total)) = self.progress.clone() else {
            return;
        };
        if self.inflight.is_empty() {
            self.progress = None;
            if started {
                self.notify(
                    "$/progress",
                    json!({ "token": token, "value": { "kind": "end", "message": "done" } }),
                );
            }
        } else if started {
            let done = total.saturating_sub(self.inflight.len());
            let value = json!({ "kind": "report", "message": format!("{done}/{total}"), "percentage": done * 100 / total.max(1) });
            self.notify("$/progress", json!({ "token": token, "value": value }));
        }
    }
}

mod requests;

#[cfg(test)]
mod tests;
