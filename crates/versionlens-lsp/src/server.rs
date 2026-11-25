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

use crate::catalog::{FileType, by_id, file_type, file_type_of, path_matches};
use crate::config::Settings;
use crate::extract::{Dep, extract};
use crate::http::Http;
use crate::osv::Query;
use crate::registry;
use crate::render;
use crate::version::Version;

const COMMANDS: &[&str] = &[
    "versionlens.update",
    "versionlens.updateAll",
    "versionlens.updateWorkspace",
    "versionlens.sort",
    "versionlens.install",
    "versionlens.refresh",
    "versionlens.clearCache",
    "versionlens.toggle",
    "versionlens.show",
    "versionlens.hide",
    "versionlens.togglePrereleases",
    "versionlens.showPrereleases",
    "versionlens.hidePrereleases",
    "versionlens.open",
    "versionlens.build",
];
/// Lookups finishing within this window share one refresh.
const COALESCE: Duration = Duration::from_millis(120);

/// A lookup: the registry, the package, the repository the manifest names, and the filters
/// the registry pages by ([`registry::filters_of`]).
type Key = (&'static str, String, Option<String>, Vec<String>);

struct Found {
    versions: Vec<Version>,
    tagged: Option<Version>,
    yanked: Vec<Version>,
    deprecated: Vec<Version>,
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
    /// Open in the editor; `false` for manifests a workspace update read from disk.
    open: bool,
}

enum Event {
    /// A lookup finished, with the generation of [`Server::lookups`] it started in.
    Fetched(Key, Result<registry::Versions, String>, Duration, u64),
    Osv(Vec<Query>, Result<Vec<Vec<String>>, String>),
    /// An install command finished: the manifest uri and the command's outcome.
    Installed(String, Result<String, String>),
}

enum Pending {
    Config,
    Progress(String),
    Ignore,
    /// A build picker: the manifest uri and requirement range the chosen build replaces.
    Build(String, Value),
    Auth(credentials::Ask),
}

#[derive(Default)]
struct Caps {
    lens_refresh: bool,
    hint_refresh: bool,
    progress: bool,
    configuration: bool,
    show_document: bool,
    /// The editor asked for the URL authentication code actions, having no commands of its own.
    auth_actions: bool,
}

struct Server {
    conn: Connection,
    runtime: tokio::runtime::Runtime,
    http: Http,
    /// The URL authentication the server stores itself; see [`credentials`].
    stored: credentials::Stored,
    settings: Settings,
    /// `enabled` and `prereleases` as last configured; the show and prerelease commands
    /// override them until the setting itself changes.
    configured: (bool, bool),
    caps: Caps,
    docs: HashMap<String, Doc>,
    cache: HashMap<Key, Entry>,
    inflight: HashSet<Key>,
    /// Bumped when lookups restart, so a lookup sent with the old credentials is not cached.
    lookups: u64,
    /// Manifests whose `updateAll` waits for their lookups to finish, with its level.
    update_after: HashMap<String, &'static str>,
    /// The level of a workspace update waiting for its lookups.
    workspace_update: Option<&'static str>,
    /// Workspace folder paths, scanned by workspace updates.
    folders: Vec<String>,
    /// The last `versionlens/status` sent per manifest, so only changes are sent.
    status: HashMap<String, Value>,
    /// The dependency list each manifest had when last installed or opened.
    installed: HashMap<String, Vec<(String, String)>>,
    /// Manifests whose install command is running.
    installing: HashSet<String>,
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
    let filters = registry::get(dep.registry).map_or_default(|r| registry::filters_of(r, &dep.req));
    (dep.registry, dep.name.clone(), dep.source.clone(), filters)
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
    serve(conn, credentials::default_path())?;
    io.join().map_err(|e| e.to_string())
}

/// Serves one editor on `conn`, keeping URL authentication in the file at `stored`.
fn serve(conn: Connection, stored: Option<std::path::PathBuf>) -> Result<(), String> {
    let (id, params) = conn.initialize_start().map_err(|e| e.to_string())?;
    let (auth_actions, commands) = credentials::advertised(&params, COMMANDS);
    let capabilities = json!({
        "textDocumentSync": { "openClose": true, "change": 2, "save": { "includeText": false } },
        "codeLensProvider": { "resolveProvider": false },
        "inlayHintProvider": true,
        "hoverProvider": true,
        "codeActionProvider": { "codeActionKinds": [render::ACTION_KIND, status::SOURCE_KIND] },
        "executeCommandProvider": { "commands": commands },
        "experimental": { "fileTypes": crate::catalog::globs() },
        "workspace": { "workspaceFolders": { "supported": true, "changeNotifications": true } },
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
        auth_actions,
    };
    let settings = params
        .get("initializationOptions")
        .map_or_default(Settings::from_json);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let mut server = Server {
        conn,
        runtime,
        http: Http::new(settings.clone())?,
        stored: credentials::Stored::at(stored),
        configured: (settings.enabled, settings.prereleases),
        settings,
        caps,
        docs: HashMap::new(),
        cache: HashMap::new(),
        inflight: HashSet::new(),
        lookups: 0,
        update_after: HashMap::new(),
        workspace_update: None,
        folders: workspace::folders(&params),
        status: HashMap::new(),
        installed: HashMap::new(),
        installing: HashSet::new(),
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
    server.reload_credentials();
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
        self.reload_credentials();
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
                    (Some(Pending::Auth(ask)), Ok(choice)) => {
                        self.answered(ask, choice.get("title").and_then(Value::as_str));
                    }
                    (Some(Pending::Build(uri, range)), Ok(choice)) => {
                        if let Some(text) = choice.get("title").and_then(Value::as_str) {
                            self.replace(&uri, &range, text);
                        }
                    }
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
                self.open(&p.text_document.uri, p.text_document.text, true);
            }
            "textDocument/didSave" => {
                let p: lsp_types::DidSaveTextDocumentParams = param(params)?;
                self.saved(p.text_document.uri.as_str());
            }
            "workspace/didChangeWorkspaceFolders" => {
                let p: lsp_types::DidChangeWorkspaceFoldersParams = param(params)?;
                self.change_folders(&p.event);
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
                self.open(&uri, text, true);
            }
            "textDocument/didClose" => {
                let p: lsp_types::DidCloseTextDocumentParams = param(params)?;
                let uri = p.text_document.uri.as_str();
                self.status.remove(uri);
                self.installed.remove(uri);
                if self.docs.remove(uri).is_some() {
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
        let mut settings = Settings::from_json(value);
        let configured = (settings.enabled, settings.prereleases);
        if configured.0 == self.configured.0 {
            settings.enabled = self.settings.enabled;
        }
        if configured.1 == self.configured.1 {
            settings.prereleases = self.settings.prereleases;
        }
        self.configured = configured;
        if settings == self.settings {
            return;
        }
        let reread = settings.files != self.settings.files
            || settings.file_types != self.settings.file_types;
        let refetch = settings.registries != self.settings.registries
            || settings.authorization != self.settings.authorization
            || settings.github_token != self.settings.github_token
            || settings.providers != self.settings.providers;
        match self.http_for(&settings) {
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
        if !self.settings.unread.is_empty() {
            let unread = self.settings.unread.join(", ");
            self.log(
                MessageType::WARNING,
                format!("not read, no counterpart in this version: {unread}; see the README"),
            );
        }
        if refetch {
            self.restart_lookups();
        }
        if reread {
            let docs: Vec<(Uri, String, bool)> = self
                .docs
                .values()
                .map(|d| (d.uri.clone(), d.text.clone(), d.open))
                .collect();
            for (uri, text, open) in docs {
                self.open(&uri, text, open);
            }
        }
        self.schedule_all();
        self.dirty = Some(Instant::now());
    }

    /// Reads a manifest the editor opened or changed, or with `open` false one read from disk.
    fn open(&mut self, uri: &Uri, text: String, open: bool) {
        let path = decode(uri.as_str().trim_start_matches("file://"));
        // Untitled buffers, git revisions, and notebook cells have no folder to resolve
        // registry settings and local paths against.
        let on_disk = uri.as_str().starts_with("file:");
        let Some(file) = self.file_type(&path, Some(&text)).filter(|_| on_disk) else {
            self.log(
                MessageType::LOG,
                format!("{path}: not a supported manifest; skipped"),
            );
            // A settings change can stop reading a manifest; its diagnostics go with it.
            if self.docs.remove(uri.as_str()).is_some() {
                self.status.remove(uri.as_str());
                self.notify(
                    "textDocument/publishDiagnostics",
                    json!({ "uri": uri, "diagnostics": [] }),
                );
            }
            return;
        };
        let started = Instant::now();
        let deps = extract(file, &text);
        let first = !self.docs.contains_key(uri.as_str());
        if first && open {
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
                open,
            },
        );
        if open && !self.installed.contains_key(uri.as_str()) {
            self.installed
                .insert(uri.as_str().to_owned(), signature(&self.docs[uri.as_str()]));
        }
        self.schedule(uri.as_str());
        self.dirty = Some(Instant::now());
    }

    /// The file type for a path: a `files` setting glob first, then the catalog, by content too
    /// when the text is at hand. Limited to the `fileTypes` setting's file type and registry
    /// ids when it lists any.
    fn file_type(&self, path: &str, text: Option<&str>) -> Option<&'static FileType> {
        let only = &self.settings.file_types;
        self.settings
            .files
            .iter()
            .filter(|(glob, _)| path_matches(glob, path))
            .find_map(|(_, id)| by_id(id))
            .or_else(|| text.map_or_else(|| file_type(path), |text| file_type_of(path, text)))
            .filter(|f| only.is_empty() || only.iter().any(|id| id == f.id || id == f.registry))
    }

    fn flush(&mut self) {
        self.dirty = None;
        for doc in self.docs.values().filter(|d| d.open) {
            let diagnostics = if self.settings.enabled {
                let severity = self.settings.update_severity;
                render::diagnostics(&self.views(doc), doc.file.insert, severity)
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
        self.send_status();
    }
}

/// A manifest's dependencies and requirements, compared across saves to decide on an install.
fn signature(doc: &Doc) -> Vec<(String, String)> {
    doc.deps
        .iter()
        .map(|d| (d.name.clone(), d.req.clone()))
        .collect()
}

mod commands;
mod credentials;
mod lookups;
mod requests;
mod status;
mod workspace;

#[cfg(test)]
mod tests;
