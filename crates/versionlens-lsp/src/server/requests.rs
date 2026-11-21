//! Editor requests and commands, answered from the documents and lookups already in memory.

use std::time::Instant;

use lsp_server::ErrorCode;
use lsp_types::{MessageType, Position, Range, Uri};
use serde_json::{Value, json};

use super::{Doc, Key, Pending, Server, key, param};
use crate::registry;
use crate::render::{self, Lines, State, View};
use crate::version::{Req, plan};

impl Server {
    pub(super) fn views<'a>(&'a self, doc: &'a Doc) -> Vec<View<'a>> {
        let lines = Lines::new(&doc.text);
        doc.deps
            .iter()
            .map(|dep| {
                let state = match self.cache.get(&key(dep)).map(|e| &e.result) {
                    None => State::Pending,
                    Some(Err(error)) => State::Failed(error.clone()),
                    Some(Ok(found)) => match registry::get(dep.registry)
                        .and_then(|r| Req::parse(&dep.req, r.bare))
                    {
                        None => State::Unreadable,
                        Some(req) => match plan(
                            &req,
                            &found.versions,
                            found.tagged.as_ref(),
                            self.settings.prereleases,
                        ) {
                            Some(plan) => State::Planned {
                                plan,
                                current: req.floor().cloned(),
                            },
                            None => State::Failed("no released versions".into()),
                        },
                    },
                };
                let vulns = self
                    .query(dep)
                    .and_then(|q| self.vulns.get(&q))
                    .map_or(&[][..], Vec::as_slice);
                View {
                    dep,
                    range: lines.range(&dep.span),
                    state,
                    vulns,
                    page: registry::page(dep.registry, &dep.name),
                }
            })
            .collect()
    }
    fn doc(&self, uri: &Uri) -> Result<&Doc, (ErrorCode, String)> {
        self.docs.get(uri.as_str()).ok_or((
            ErrorCode::InvalidParams,
            format!("{} is not an open manifest", uri.as_str()),
        ))
    }

    /// Editor feature requests answer `null` for a file that is not an open manifest, since
    /// editors such as Zed send every file of a language; commands report it instead.
    pub(super) fn handle(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Value, (ErrorCode, String)> {
        let bad = |e: String| (ErrorCode::InvalidParams, e);
        let on = self.settings.enabled;
        let within = |r: &Range, p: &Position| r.start <= *p && *p <= r.end;
        let result = match method {
            "textDocument/codeLens" => {
                let p: lsp_types::CodeLensParams = param(params).map_err(bad)?;
                let Some(doc) = self.docs.get(p.text_document.uri.as_str()) else {
                    return Ok(Value::Null);
                };
                let show = on && self.settings.code_lens;
                json!(if show {
                    render::lenses(&doc.uri, &self.views(doc), doc.file.insert)
                } else {
                    Vec::new()
                })
            }
            "textDocument/inlayHint" => {
                let p: lsp_types::InlayHintParams = param(params).map_err(bad)?;
                let Some(doc) = self.docs.get(p.text_document.uri.as_str()) else {
                    return Ok(Value::Null);
                };
                let views: Vec<View<'_>> = self
                    .views(doc)
                    .into_iter()
                    .filter(|v| p.range.start <= v.range.end && v.range.start <= p.range.end)
                    .collect();
                json!(if on && self.settings.inlay_hints {
                    render::hints(&views)
                } else {
                    Vec::new()
                })
            }
            "textDocument/hover" => {
                let p: lsp_types::HoverParams = param(params).map_err(bad)?;
                let Some(doc) = self
                    .docs
                    .get(p.text_document_position_params.text_document.uri.as_str())
                else {
                    return Ok(Value::Null);
                };
                let at = p.text_document_position_params.position;
                // Prefer the dependency under the cursor, then any dependency on the cursor's line.
                let views = self.views(doc);
                let on_line =
                    |v: &&View<'_>| v.range.start.line <= at.line && at.line <= v.range.end.line;
                let view = views
                    .iter()
                    .find(|v| within(&v.range, &at))
                    .or_else(|| views.iter().find(on_line));
                json!(view.filter(|_| on).map(render::hover))
            }
            "textDocument/codeAction" => {
                let p: lsp_types::CodeActionParams = param(params).map_err(bad)?;
                let Some(doc) = self.docs.get(p.text_document.uri.as_str()) else {
                    return Ok(Value::Null);
                };
                // The light bulb follows the line, so a cursor on the package name offers fixes too.
                let views: Vec<View<'_>> = self
                    .views(doc)
                    .into_iter()
                    .filter(|v| {
                        p.range.start.line <= v.range.end.line
                            && v.range.start.line <= p.range.end.line
                    })
                    .collect();
                json!(if on && render::wants_actions(p.context.only.as_deref()) {
                    render::actions(&doc.uri, &views, doc.file.insert)
                } else {
                    Vec::new()
                })
            }
            "workspace/executeCommand" => {
                let p: lsp_types::ExecuteCommandParams = param(params).map_err(bad)?;
                self.command(&p.command, &p.arguments)?;
                Value::Null
            }
            _ => {
                return Err((
                    ErrorCode::MethodNotFound,
                    format!("unsupported request {method}"),
                ));
            }
        };
        Ok(result)
    }

    /// Runs each deferred `updateAll` whose manifest has no lookups left.
    pub(super) fn run_deferred_updates(&mut self) {
        let ready: Vec<String> = self
            .update_after
            .iter()
            .filter(|uri| {
                self.docs
                    .get(*uri)
                    .is_none_or(|doc| !doc.deps.iter().any(|d| self.inflight.contains(&key(d))))
            })
            .cloned()
            .collect();
        for uri in ready {
            self.update_after.remove(&uri);
            self.update_all(&uri);
        }
    }

    fn update_all(&mut self, uri: &str) {
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
        let edit = render::update_all(&doc.uri, &views, doc.file.insert);
        let message = match failed {
            0 => format!("{}: every dependency is up to date", doc.path),
            n => format!(
                "{}: {n} dependencies could not be looked up; see the log",
                doc.path
            ),
        };
        match edit {
            Some(edit) => self.request("workspace/applyEdit", json!({ "label": "Update all dependencies", "edit": edit }), Pending::Ignore),
            None => self.notify("window/showMessage", json!({ "type": if failed == 0 { MessageType::INFO } else { MessageType::WARNING }, "message": message })),
        }
    }

    fn command(&mut self, command: &str, args: &[Value]) -> Result<(), (ErrorCode, String)> {
        let bad = |e: String| (ErrorCode::InvalidParams, e);
        let uri = args.first().cloned().map(param::<Uri>).transpose();
        self.log(MessageType::LOG, format!("command {command}"));
        match command {
            "versionlens.update" => {
                let [uri, range, text] = args else {
                    return Err(bad("expected [uri, range, text]".into()));
                };
                let edit = json!({ "changes": { uri.as_str().unwrap_or_default(): [{ "range": range, "newText": text }] } });
                self.request(
                    "workspace/applyEdit",
                    json!({ "label": "Update dependency", "edit": edit }),
                    Pending::Ignore,
                );
            }
            "versionlens.updateAll" => {
                let uri = uri
                    .map_err(bad)?
                    .ok_or_else(|| bad("expected the manifest uri".into()))?;
                let doc = self.doc(&uri)?;
                let waiting = doc
                    .deps
                    .iter()
                    .filter(|d| self.inflight.contains(&key(d)))
                    .count();
                if waiting == 0 {
                    self.update_all(uri.as_str());
                } else {
                    // Updating now would skip the dependencies still being looked up.
                    self.log(
                        MessageType::INFO,
                        format!("{}: updating after {waiting} lookups", doc.path),
                    );
                    self.update_after.insert(uri.as_str().to_owned());
                }
            }
            "versionlens.refresh" | "versionlens.clearCache" => {
                match uri
                    .map_err(bad)?
                    .filter(|_| command == "versionlens.refresh")
                {
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
            "versionlens.toggle" => {
                self.settings.enabled = !self.settings.enabled;
                self.log(
                    MessageType::INFO,
                    format!(
                        "VersionLens {}",
                        if self.settings.enabled {
                            "shown"
                        } else {
                            "hidden"
                        }
                    ),
                );
                self.dirty = Some(Instant::now());
            }
            "versionlens.open" => {
                let url = args
                    .first()
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad("expected a URL".into()))?
                    .to_owned();
                if self.caps.show_document {
                    self.request(
                        "window/showDocument",
                        json!({ "uri": url, "external": true }),
                        Pending::Ignore,
                    );
                } else {
                    self.notify(
                        "window/showMessage",
                        json!({ "type": MessageType::INFO, "message": url }),
                    );
                }
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
}
