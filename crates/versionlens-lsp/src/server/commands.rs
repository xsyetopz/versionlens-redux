//! `workspace/executeCommand`: the commands every editor offers, from per-dependency updates to
//! workspace updates, sorting, install commands, and the show and prerelease switches.

use std::time::Instant;

use lsp_server::ErrorCode;
use lsp_types::{MessageType, Uri};
use serde_json::{Value, json};

use super::{Event, Key, Pending, Server, key, signature};
use crate::render::{self, LEVELS, State};
use crate::sort;

type Failure = (ErrorCode, String);

fn bad(text: impl Into<String>) -> Failure {
    (ErrorCode::InvalidParams, text.into())
}

/// The manifest and level among a command's arguments, in any order: editors that inject the
/// active file add its uri in front of the arguments a menu passes.
fn targets(args: &[Value]) -> (Option<Uri>, Option<&'static str>) {
    let strings = || args.iter().filter_map(Value::as_str);
    let level = strings().find_map(|s| LEVELS.iter().find(|l| **l == s).copied());
    let uri = strings()
        .filter(|s| s.contains(':'))
        .find_map(|s| s.parse::<Uri>().ok());
    (uri, level)
}

impl Server {
    pub(super) fn command(&mut self, command: &str, args: &[Value]) -> Result<(), Failure> {
        self.log(MessageType::LOG, format!("command {command}"));
        let (uri, level) = targets(args);
        let need_uri = || uri.clone().ok_or_else(|| bad("expected the manifest uri"));
        match command {
            "versionlens.update" => {
                let [uri, range, text] = args else {
                    return Err(bad("expected [uri, range, text]"));
                };
                let edit = json!({ "changes": { uri.as_str().unwrap_or_default(): [{ "range": range, "newText": text }] } });
                self.request(
                    "workspace/applyEdit",
                    json!({ "label": "Update dependency", "edit": edit }),
                    Pending::Ignore,
                );
            }
            "versionlens.updateAll" => {
                let uri = need_uri()?;
                let level = level.unwrap_or("latest");
                let doc = self.doc(&uri)?;
                let waiting = doc
                    .deps
                    .iter()
                    .filter(|d| self.inflight.contains(&key(d)))
                    .count();
                if waiting == 0 {
                    self.update_all(uri.as_str(), level);
                } else {
                    // Updating now would skip the dependencies still being looked up.
                    self.log(
                        MessageType::INFO,
                        format!("{}: updating after {waiting} lookups", doc.path),
                    );
                    self.update_after.insert(uri.as_str().to_owned(), level);
                }
            }
            "versionlens.updateWorkspace" => {
                self.update_workspace(level.unwrap_or("latest"))
                    .map_err(|e| (ErrorCode::RequestFailed, e))?;
            }
            "versionlens.sort" => self.sort(&need_uri()?)?,
            "versionlens.install" => {
                let uri = need_uri()?;
                self.doc(&uri)?;
                self.install(uri.as_str())
                    .map_err(|e| (ErrorCode::RequestFailed, e))?;
            }
            "versionlens.refresh" | "versionlens.clearCache" => {
                match uri.filter(|_| command == "versionlens.refresh") {
                    Some(uri) => {
                        let keys: Vec<Key> = self.doc(&uri)?.deps.iter().map(key).collect();
                        for key in keys {
                            self.cache.remove(&key);
                        }
                    }
                    None => self.cache.clear(),
                }
                self.vulns.clear();
                self.osv_warned = false;
                self.schedule_all();
                self.dirty = Some(Instant::now());
            }
            "versionlens.toggle"
            | "versionlens.show"
            | "versionlens.hide"
            | "versionlens.togglePrereleases"
            | "versionlens.showPrereleases"
            | "versionlens.hidePrereleases" => self.switch(command),
            "versionlens.open" => {
                let url = args.first().and_then(Value::as_str);
                self.open_url(url.ok_or_else(|| bad("expected a URL"))?);
            }
            _ => {
                return Err((
                    ErrorCode::MethodNotFound,
                    format!("unknown command {command}"),
                ));
            }
        }
        Ok(())
    }

    /// Shows, hides, or toggles the versions or the prereleases.
    fn switch(&mut self, command: &str) {
        let name = command.trim_start_matches("versionlens.");
        let (what, flag) = if name.ends_with("Prereleases") {
            ("prereleases", &mut self.settings.prereleases)
        } else {
            ("versions", &mut self.settings.enabled)
        };
        *flag = match name.trim_end_matches("Prereleases") {
            "show" => true,
            "hide" => false,
            _ => !*flag,
        };
        let state = if *flag { "shown" } else { "hidden" };
        self.log(MessageType::INFO, format!("{what} {state}"));
        self.dirty = Some(Instant::now());
    }

    /// Opens a registry page in the browser, or shows its URL when the editor cannot.
    fn open_url(&mut self, url: &str) {
        if self.caps.show_document {
            self.request(
                "window/showDocument",
                json!({ "uri": url, "external": true }),
                Pending::Ignore,
            );
        } else {
            self.show(MessageType::INFO, url);
        }
    }

    fn show(&self, kind: MessageType, text: impl Into<String>) {
        let text = text.into();
        self.notify(
            "window/showMessage",
            json!({ "type": kind, "message": text }),
        );
    }

    /// Runs each deferred `updateAll` whose manifest has no lookups left, then a waiting
    /// workspace update.
    pub(super) fn run_deferred_updates(&mut self) {
        let ready: Vec<(String, &'static str)> = self
            .update_after
            .iter()
            .filter(|(uri, _)| {
                self.docs
                    .get(*uri)
                    .is_none_or(|doc| !doc.deps.iter().any(|d| self.inflight.contains(&key(d))))
            })
            .map(|(uri, level)| (uri.clone(), *level))
            .collect();
        for (uri, level) in ready {
            self.update_after.remove(&uri);
            self.update_all(&uri, level);
        }
        self.finish_workspace_update();
    }

    fn update_all(&mut self, uri: &str, level: &str) {
        let Some(doc) = self.docs.get(uri) else {
            return self.log(
                MessageType::WARNING,
                format!("{uri} closed before its update"),
            );
        };
        let views = self.views(doc);
        let failed = views
            .iter()
            .filter(|v| matches!(v.state, State::Failed(_)))
            .count();
        let edits = render::updates(&views, doc.file.insert, level);
        if edits.is_empty() {
            let text = match failed {
                0 => format!("{}: no {level} updates", doc.path),
                n => format!(
                    "{}: {n} dependencies could not be looked up; see the log",
                    doc.path
                ),
            };
            let kind = if failed == 0 {
                MessageType::INFO
            } else {
                MessageType::WARNING
            };
            return self.show(kind, text);
        }
        let edit = json!({ "changes": { uri: edits } });
        self.request(
            "workspace/applyEdit",
            json!({ "label": format!("Update dependencies ({level})"), "edit": edit }),
            Pending::Ignore,
        );
    }

    fn sort(&mut self, uri: &Uri) -> Result<(), Failure> {
        let doc = self.doc(uri)?;
        let commas = matches!(doc.file.format, crate::flatten::Format::Json);
        let lines = render::Lines::new(&doc.text);
        let edits: Vec<Value> = sort::runs(&doc.text, &doc.deps, commas)
            .into_iter()
            .filter(|run| doc.text[run.span.clone()] != run.sorted)
            .map(|run| json!({ "range": lines.range(&run.span), "newText": run.sorted }))
            .collect();
        if edits.is_empty() {
            let text = format!("{}: dependencies are already sorted", doc.path);
            self.show(MessageType::INFO, text);
            return Ok(());
        }
        let edit = json!({ "changes": { uri.as_str(): edits } });
        self.request(
            "workspace/applyEdit",
            json!({ "label": "Sort dependencies", "edit": edit }),
            Pending::Ignore,
        );
        Ok(())
    }

    /// Runs the configured install command when a save changed the manifest's dependencies.
    pub(super) fn saved(&mut self, uri: &str) {
        let Some(doc) = self.docs.get(uri) else {
            return;
        };
        let now = signature(doc);
        if self.installed.get(uri) == Some(&now) || self.install_command(doc).is_none() {
            return;
        }
        if let Err(error) = self.install(uri) {
            self.show(MessageType::ERROR, error);
        }
    }

    /// Starts the manifest's install command in its directory; the result arrives as an event.
    fn install(&mut self, uri: &str) -> Result<(), String> {
        let doc = &self.docs[uri];
        let Some(command) = self.install_command(doc).map(str::to_owned) else {
            return Err(format!(
                "{}: no install command; set versionlens.install for \"{}\" or \"{}\"",
                doc.path, doc.file.id, doc.file.registry
            ));
        };
        if !self.installing.insert(uri.to_owned()) {
            return Err(format!("{}: {command} is already running", doc.path));
        }
        self.installed.insert(uri.to_owned(), signature(doc));
        let dir = std::path::Path::new(&doc.path)
            .parent()
            .map_or_default(std::path::Path::to_path_buf);
        self.log(
            MessageType::INFO,
            format!("{}: running {command}", dir.display()),
        );
        let (tx, uri) = (self.events.0.clone(), uri.to_owned());
        std::thread::spawn(move || {
            let shell = if cfg!(windows) {
                ("cmd", "/C")
            } else {
                ("sh", "-c")
            };
            let output = std::process::Command::new(shell.0)
                .args([shell.1, &command])
                .current_dir(&dir)
                .output();
            let result = match output {
                Err(error) => Err(format!("{command}: {error}")),
                Ok(out) => {
                    let text = String::from_utf8_lossy(&out.stdout).into_owned()
                        + &String::from_utf8_lossy(&out.stderr);
                    if out.status.success() {
                        Ok(format!("{command}: done\n{text}"))
                    } else {
                        Err(format!("{command}: {}\n{text}", out.status))
                    }
                }
            };
            drop(tx.send(Event::Installed(uri, result)));
        });
        self.dirty.get_or_insert_with(Instant::now);
        Ok(())
    }

    pub(super) fn install_done(&mut self, uri: &str, result: Result<String, String>) {
        self.installing.remove(uri);
        self.dirty.get_or_insert_with(Instant::now);
        match result {
            Ok(text) => {
                self.log(MessageType::INFO, text.trim_end());
                let first = text.lines().next().unwrap_or_default().to_owned();
                self.show(MessageType::INFO, format!("VersionLens: {first}"));
            }
            Err(text) => {
                self.log(MessageType::ERROR, text.trim_end());
                let first = text.lines().next().unwrap_or_default().to_owned();
                self.show(
                    MessageType::ERROR,
                    format!("VersionLens: {first}; see the log"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests;
