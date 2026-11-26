//! Workspace folders and workspace-wide updates: every manifest under the folders is read from
//! disk, looked up, and updated in one edit.

use std::path::{Path, PathBuf};

use lsp_types::{MessageType, Uri, WorkspaceFoldersChangeEvent};
use serde_json::{Value, json};

use super::{Pending, Server, decode};

/// Directories that hold installed or generated packages rather than the project's manifests.
const SKIP: &[&str] = &[
    "node_modules",
    "target",
    "vendor",
    "dist",
    "build",
    "out",
    "bin",
    "obj",
    "Pods",
    "venv",
    "__pycache__",
    "bower_components",
    "jspm_packages",
];
/// Hidden directories that do hold manifests.
const HIDDEN_KEEP: &[&str] = &[".github", ".cargo", ".config"];
/// Bounds a scan of a very large tree; a workspace update reports when it stops early.
const MAX_FILES: usize = 20_000;

fn path_of(uri: &str) -> Option<String> {
    uri.strip_prefix("file://").map(decode)
}

/// A `file://` URI for an absolute path, percent-encoding what URIs reserve.
pub(super) fn file_uri(path: &str) -> String {
    let mut uri = String::from(if path.starts_with('/') {
        "file://"
    } else {
        "file:///"
    });
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~:".contains(&byte) {
            uri.push(char::from(byte));
        } else {
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            uri.push('%');
            uri.push(char::from(HEX[usize::from(byte >> 4)]));
            uri.push(char::from(HEX[usize::from(byte & 15)]));
        }
    }
    uri
}

/// The workspace folder paths in `initialize` params: `workspaceFolders`, else `rootUri`.
pub(super) fn folders(params: &Value) -> Vec<String> {
    let listed: Vec<String> = params
        .get("workspaceFolders")
        .and_then(Value::as_array)
        .map_or_default(|list| {
            list.iter()
                .filter_map(|f| path_of(f.get("uri")?.as_str()?))
                .collect()
        });
    if listed.is_empty() {
        params
            .get("rootUri")
            .and_then(Value::as_str)
            .and_then(path_of)
            .into_iter()
            .collect()
    } else {
        listed
    }
}

/// Collects files under `dir` into `out`, skipping package and build directories and every
/// directory or file an `exclude` glob matches.
fn walk(dir: &Path, exclude: &[String], out: &mut Vec<PathBuf>, visited: &mut usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if *visited >= MAX_FILES {
            return;
        }
        *visited += 1;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // `file_type` does not follow symlinks, so linked directories cannot loop.
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        let text = path.to_string_lossy();
        if exclude
            .iter()
            .any(|glob| crate::catalog::path_matches(glob, &text))
        {
            continue;
        }
        if kind.is_dir() {
            let hidden = name.starts_with('.') && !HIDDEN_KEEP.contains(&name.as_ref());
            if !hidden && !SKIP.contains(&name.as_ref()) {
                walk(&path, exclude, out, visited);
            }
        } else if kind.is_file() {
            out.push(path);
        }
    }
}

impl Server {
    pub(super) fn change_folders(&mut self, event: &WorkspaceFoldersChangeEvent) {
        for gone in &event.removed {
            if let Some(path) = path_of(gone.uri.as_str()) {
                self.folders.retain(|f| *f != path);
            }
        }
        self.folders
            .extend(event.added.iter().filter_map(|f| path_of(f.uri.as_str())));
    }

    /// Reads every manifest under the workspace folders and updates them once their lookups end.
    pub(super) fn update_workspace(&mut self, level: &'static str) -> Result<(), String> {
        if self.folders.is_empty() {
            return Err("the editor reported no workspace folder".into());
        }
        let mut files = Vec::new();
        let mut visited = 0;
        for folder in &self.folders {
            let exclude = &self.settings.workspace_exclude;
            walk(Path::new(folder), exclude, &mut files, &mut visited);
        }
        if visited >= MAX_FILES {
            self.log(
                MessageType::WARNING,
                format!("workspace update: stopped scanning after {MAX_FILES} files"),
            );
        }
        let mut read = 0;
        for file in files {
            let path = file.to_string_lossy().replace('\\', "/");
            // By name only: reading every JSON file to test its content would slow the scan.
            if self.file_type(&path, None).is_none() {
                continue;
            }
            let uri = file_uri(&path);
            if self.docs.contains_key(&uri) {
                read += 1;
                continue;
            }
            let Ok(parsed) = uri.parse::<Uri>() else {
                self.log(MessageType::WARNING, format!("{path}: not a valid uri"));
                continue;
            };
            match std::fs::read_to_string(&file) {
                Ok(text) => {
                    self.open(&parsed, text, false);
                    read += 1;
                }
                Err(error) => self.log(MessageType::WARNING, format!("{path}: {error}")),
            }
        }
        self.log(
            MessageType::INFO,
            format!("workspace update ({level}): {read} manifests"),
        );
        self.workspace_update = Some(level);
        self.run_deferred_updates();
        Ok(())
    }

    /// Applies a waiting workspace update once no manifest has lookups in flight.
    pub(super) fn finish_workspace_update(&mut self) {
        let Some(level) = self.workspace_update else {
            return;
        };
        if !self.inflight.is_empty() {
            return;
        }
        self.workspace_update = None;
        let mut changes = serde_json::Map::new();
        let mut count = 0;
        for doc in self.docs.values() {
            let edits = crate::render::updates(&self.views(doc), doc.file.insert, level);
            if !edits.is_empty() {
                count += edits.len();
                changes.insert(doc.uri.as_str().to_owned(), json!(edits));
            }
        }
        self.docs.retain(|_, doc| doc.open);
        if changes.is_empty() {
            let message = format!("VersionLens: no {level} updates in the workspace");
            self.notify(
                "window/showMessage",
                json!({ "type": MessageType::INFO, "message": message }),
            );
            return;
        }
        self.log(
            MessageType::INFO,
            format!(
                "workspace update ({level}): {count} dependencies in {} manifests",
                changes.len()
            ),
        );
        let label = format!("Update workspace dependencies ({level})");
        self.request(
            "workspace/applyEdit",
            json!({ "label": label, "edit": { "changes": changes } }),
            Pending::Ignore,
        );
    }
}
