//! Editor requests and commands, answered from the documents and lookups already in memory.

use lsp_server::ErrorCode;
use lsp_types::{Position, Range, Uri};
use serde_json::{Value, json};

use super::{Doc, Server, key, param};
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
    pub(super) fn doc(&self, uri: &Uri) -> Result<&Doc, (ErrorCode, String)> {
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
                let only = p.context.only.as_deref();
                let mut actions = if on && render::wants(render::ACTION_KIND, only) {
                    render::actions(&doc.uri, &views, doc.file.insert)
                } else {
                    Vec::new()
                };
                if render::wants(super::status::SOURCE_KIND, only) {
                    actions.extend(self.source_actions(doc));
                }
                json!(actions)
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
}
