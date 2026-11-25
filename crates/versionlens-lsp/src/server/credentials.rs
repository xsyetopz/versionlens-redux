//! URL authentication the server keeps itself, for editors with neither a secret store nor a text
//! prompt, such as Zed. The headers live in a JSON file only its owner can read, keyed by registry
//! URL prefix, and replace a settings header for the same prefix as VS Code's secret storage does.
//! The Add and Remove code actions ask through `window/showMessageRequest` buttons; Add then opens
//! the file at the new entry, since an editor without a text prompt still edits files.

use std::collections::HashMap;
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use lsp_server::ErrorCode;
use lsp_types::{MessageType, Uri};
use serde_json::{Map, Value, json};

use super::{Pending, Server};
use crate::config::Settings;
use crate::http::Http;
use crate::project::base64;
use crate::registry;

/// The ways a registry authenticates, as VS Code's Add command offers them.
pub(super) const KINDS: [&str; 3] = ["Basic (user and password)", "Bearer token", "Header value"];

/// The commands behind the two code actions, advertised only to editors that ask for them.
pub(super) const COMMANDS: [&str; 2] = [
    "versionlens.addAuthentication",
    "versionlens.removeAuthentication",
];

/// The stored file and the headers it held when last read.
#[derive(Default)]
pub(super) struct Stored {
    path: Option<PathBuf>,
    read: Option<SystemTime>,
    headers: HashMap<String, String>,
}

impl Stored {
    /// Nothing read yet from the file at `path`.
    pub(super) fn at(path: Option<PathBuf>) -> Self {
        Self {
            path,
            ..Self::default()
        }
    }
}

/// Whether the editor's `initializationOptions` ask for the authentication code actions, and the
/// commands to advertise: VS Code registers each one, which would clash with its own Add and
/// Remove, so they join `base` only on request.
pub(super) fn advertised(params: &Value, base: &[&'static str]) -> (bool, Vec<&'static str>) {
    let wanted = params
        .pointer("/initializationOptions/authenticationActions")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut commands = base.to_vec();
    if wanted {
        commands.extend(COMMANDS);
    }
    (wanted, commands)
}

/// A question the server asked with `window/showMessageRequest`.
pub(super) enum Ask {
    /// Which kind of header to add, for the URL prefix the new entry starts with.
    Kind(String),
    /// Which stored URL prefix to forget.
    Forget,
}

/// `versionlens/authorization.json` in the user's configuration directory: `%APPDATA%` on
/// Windows, `$XDG_CONFIG_HOME` or `~/.config` elsewhere.
pub(super) fn default_path() -> Option<PathBuf> {
    let var = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let base = if cfg!(windows) {
        var("APPDATA")
    } else {
        var("XDG_CONFIG_HOME").or_else(|| var("HOME").map(|home| home.join(".config")))
    };
    base.map(|dir| dir.join("versionlens").join("authorization.json"))
}

/// The entries of the stored file; a missing or blank file has none.
fn read(path: &Path) -> Result<Map<String, Value>, String> {
    let text = match std::fs::read_to_string(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Map::new()),
        Err(error) => return Err(format!("{}: {error}", path.display())),
        Ok(text) => text,
    };
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str(&text) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err(format!("{}: not a JSON object", path.display())),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

/// Replaces the stored file with `entries`, readable and writable only by its owner.
fn write(path: &Path, entries: &Map<String, Value>) -> Result<String, String> {
    let fail = |error: std::io::Error| format!("{}: {error}", path.display());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path).map_err(fail)?;
    // The mode above applies only to a new file; an older one keeps its own until this.
    #[cfg(unix)]
    file.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))
        .map_err(fail)?;
    let text = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())? + "\n";
    file.write_all(text.as_bytes()).map_err(fail)?;
    Ok(text)
}

/// The header an entry sends: a string as written, `{"token"}` as Bearer, and
/// `{"user", "password"}` as Basic. None while a part is still blank.
pub(super) fn header(value: &Value) -> Option<String> {
    let part = |key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
    };
    match value {
        Value::String(text) => Some(text.trim().to_owned()).filter(|t| !t.is_empty()),
        _ if let Some(token) = part("token") => Some(format!("Bearer {}", token.trim())),
        _ => {
            let pair = format!("{}:{}", part("user")?.trim(), part("password")?);
            Some(format!("Basic {}", base64(pair.as_bytes())))
        }
    }
}

/// The blank entry for one of [`KINDS`].
fn template(kind: &str) -> Value {
    match kind {
        k if k == KINDS[0] => json!({ "user": "", "password": "" }),
        k if k == KINDS[1] => json!({ "token": "" }),
        _ => json!(""),
    }
}

/// Where the cursor goes in the written `text` of the file: inside the first blank string of
/// the `url` entry, or on the URL itself when it is only the scheme left to complete.
pub(super) fn cursor(text: &str, url: &str) -> Value {
    let quoted = serde_json::to_string(url).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.starts_with(&format!("  {quoted}:")))
        .unwrap_or_default();
    let width = |s: &str| s.encode_utf16().count();
    let (line, from, to) = if url.ends_with("://") {
        (start, 3, 3 + width(url))
    } else {
        lines
            .iter()
            .enumerate()
            .skip(start)
            .find_map(|(n, l)| {
                l.find("\"\"")
                    .map(|at| (n, width(&l[..at]) + 1, width(&l[..at]) + 1))
            })
            .unwrap_or((start, 3, 3))
    };
    let at = |character| json!({ "line": line, "character": character });
    json!({ "start": at(from), "end": at(to) })
}

/// The URL prefix of a registry: its configured or built-in URL up to the last `/` before its
/// first placeholder, or the bare scheme when that leaves no host.
fn prefix(settings: &Settings, id: &str) -> String {
    let template = settings
        .registries
        .get(id)
        .map(String::as_str)
        .or_else(|| registry::get(id).map(|r| r.url))
        .unwrap_or_default();
    let fixed = &template[..template.find('{').unwrap_or(template.len())];
    match fixed.rfind('/') {
        Some(end) if fixed[..end].contains("://") => fixed[..=end].to_owned(),
        _ => "https://".to_owned(),
    }
}

impl Server {
    /// The HTTP client for `settings` with the stored headers over its `authorization`.
    pub(super) fn http_for(&self, settings: &Settings) -> Result<Http, String> {
        let mut merged = settings.clone();
        merged.authorization.extend(self.stored.headers.clone());
        Http::new(merged)
    }

    /// Rereads the stored file when it changed since the last read, which also catches an edit
    /// saved through another server or outside the editor, then looks the packages up again.
    pub(super) fn reload_credentials(&mut self) {
        let Some(path) = self.stored.path.clone() else {
            return;
        };
        let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        if modified == self.stored.read {
            return;
        }
        self.stored.read = modified;
        let headers = match read(&path) {
            Ok(entries) => entries
                .iter()
                .filter_map(|(url, value)| header(value).map(|h| (url.clone(), h)))
                .collect(),
            Err(error) => {
                return self.log(MessageType::ERROR, format!("url authentication: {error}"));
            }
        };
        if headers == self.stored.headers {
            return;
        }
        self.stored.headers = headers;
        let text = format!(
            "url authentication: {} URLs from {}",
            self.stored.headers.len(),
            path.display()
        );
        self.log(MessageType::INFO, text);
        match self.http_for(&self.settings) {
            Ok(http) => self.http = http,
            Err(error) => self.log(MessageType::ERROR, format!("url authentication: {error}")),
        }
        self.restart_lookups();
        self.schedule_all();
        self.dirty = Some(Instant::now());
    }

    fn stored_path(&self) -> Result<PathBuf, (ErrorCode, String)> {
        self.stored.path.clone().ok_or_else(|| {
            let need = if cfg!(windows) { "APPDATA" } else { "HOME" };
            (
                ErrorCode::RequestFailed,
                format!("no configuration directory; set {need}"),
            )
        })
    }

    /// `versionlens.addAuthentication`: asks for the kind of header, for the URL prefix of the
    /// manifest's registry when a manifest is named.
    pub(super) fn ask_kind(&mut self, uri: Option<&Uri>) -> Result<(), (ErrorCode, String)> {
        self.stored_path()?;
        let registry = uri
            .and_then(|u| self.docs.get(u.as_str()))
            .map(|d| d.file.registry);
        let url = registry.map_or_else(|| "https://".to_owned(), |id| prefix(&self.settings, id));
        let actions: Vec<Value> = KINDS.iter().map(|k| json!({ "title": k })).collect();
        let message = format!("How does {url} authenticate?");
        let params = json!({ "type": MessageType::INFO, "message": message, "actions": actions });
        self.request(
            "window/showMessageRequest",
            params,
            Pending::Auth(Ask::Kind(url)),
        );
        Ok(())
    }

    /// `versionlens.removeAuthentication`: asks which stored URL prefix to forget.
    pub(super) fn ask_forget(&mut self) -> Result<(), (ErrorCode, String)> {
        let path = self.stored_path()?;
        let urls: Vec<Value> = read(&path)
            .map_err(|e| (ErrorCode::RequestFailed, e))?
            .keys()
            .map(|url| json!({ "title": url }))
            .collect();
        if urls.is_empty() {
            self.show(
                MessageType::INFO,
                "VersionLens: no stored url authentication",
            );
            return Ok(());
        }
        let params = json!({ "type": MessageType::INFO, "message": "Forget the authentication of which URL?", "actions": urls });
        self.request(
            "window/showMessageRequest",
            params,
            Pending::Auth(Ask::Forget),
        );
        Ok(())
    }

    /// Acts on the button chosen for `ask`; a dismissed prompt changes nothing.
    pub(super) fn answered(&mut self, ask: Ask, choice: Option<&str>) {
        let Some(choice) = choice else {
            return self.log(MessageType::LOG, "url authentication: prompt dismissed");
        };
        let Some(path) = self.stored.path.clone() else {
            return;
        };
        let result = read(&path).and_then(|mut entries| match ask {
            Ask::Kind(url) => {
                entries.insert(url.clone(), template(choice));
                let text = write(&path, &entries)?;
                Ok((
                    format!("VersionLens: fill in the {choice} for {url} and save"),
                    Some((url, text)),
                ))
            }
            Ask::Forget => {
                entries.remove(choice);
                write(&path, &entries)?;
                Ok((
                    format!("VersionLens: authentication removed for {choice}"),
                    None,
                ))
            }
        });
        match result {
            Ok((message, open)) => {
                if let Some((url, text)) = open {
                    self.open_stored(&path, &url, &text);
                }
                self.show(MessageType::INFO, message);
                self.reload_credentials();
            }
            Err(error) => self.show(MessageType::ERROR, format!("VersionLens: {error}")),
        }
    }

    /// Opens the stored file at the new entry, or names the file when the editor cannot open it.
    fn open_stored(&mut self, path: &Path, url: &str, text: &str) {
        if !self.caps.show_document {
            let text = format!("VersionLens: edit {} to finish", path.display());
            return self.show(MessageType::INFO, text);
        }
        let uri = super::workspace::file_uri(&path.display().to_string().replace('\\', "/"));
        let params = json!({ "uri": uri, "takeFocus": true, "selection": cursor(text, url) });
        self.request("window/showDocument", params, Pending::Ignore);
    }
}

#[cfg(test)]
mod tests;
