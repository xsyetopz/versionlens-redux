//! Editor requests and commands, answered from the documents and lookups already in memory.

use lsp_server::ErrorCode;
use lsp_types::{Position, Range, Uri};
use serde_json::{Value, json};
use std::borrow::Cow;

use super::workspace::file_uri;
use super::{Doc, Server, key, param};
use crate::extract::{DIRECTORY, Dep, PROJECT, SOURCE, local_path};
use crate::registry;
use crate::render::{self, Lines, State, View};
use crate::version::{Req, Version, increments, plan};

impl Server {
    /// The versions without the prereleases the registry's `prereleaseTags` leave out.
    fn tagged_prereleases<'v>(
        &self,
        registry: &str,
        versions: &'v [Version],
    ) -> Cow<'v, [Version]> {
        let tags = self.settings.prerelease_tags_for(registry);
        if tags.is_empty() {
            return Cow::Borrowed(versions);
        }
        versions
            .iter()
            .filter(|v| {
                let tag = v.tag.to_ascii_lowercase();
                !v.is_prerelease() || tags.iter().any(|t| tag.contains(&t.to_ascii_lowercase()))
            })
            .cloned()
            .collect()
    }

    fn state(&self, doc: &Doc, dep: &Dep) -> State {
        if dep.registry == DIRECTORY {
            return directory(&doc.path, &dep.req);
        }
        if dep.registry == SOURCE {
            return State::Source(dep.source.clone().unwrap_or_default());
        }
        if dep.registry == PROJECT {
            return State::Own(
                increments(&dep.req)
                    .into_iter()
                    .map(|(label, text)| (format!("{label} {text}"), text))
                    .collect(),
            );
        }
        let found = match self.cache.get(&key(dep)).map(|e| &e.result) {
            None => return State::Pending,
            Some(Err(error)) => return State::Failed(error.clone()),
            Some(Ok(found)) => found,
        };
        let Some(req) = registry::get(dep.registry).and_then(|r| Req::parse(&dep.req, r.bare))
        else {
            return State::Unreadable;
        };
        let versions = self.tagged_prereleases(dep.registry, &found.versions);
        let tagged = found.tagged.as_ref();
        match plan(&req, &versions, tagged, self.settings.prereleases) {
            Some(mut plan) => {
                plan.mark(&req, &found.yanked, &found.deprecated);
                State::Planned {
                    plan: Box::new(plan),
                    current: req.floor().cloned(),
                }
            }
            None => State::Failed("no released versions".into()),
        }
    }

    pub(super) fn views<'a>(&'a self, doc: &'a Doc) -> Vec<View<'a>> {
        let lines = Lines::new(&doc.text);
        doc.deps
            .iter()
            .map(|dep| {
                let state = self.state(doc, dep);
                let vulns = self
                    .query(dep)
                    .and_then(|q| self.vulns.get(&q))
                    .map_or(&[][..], Vec::as_slice);
                View {
                    dep,
                    range: lines.range(&dep.span),
                    state,
                    vulns,
                    page: page(dep),
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
                if !show {
                    return Ok(json!([]));
                }
                let (views, glyphs) = (self.views(doc), &self.settings.indicators);
                let insert = doc.file.insert;
                let stats = self.settings.stats;
                let head = stats.then(|| render::stats(&doc.uri, &views, insert, glyphs));
                let mut lenses: Vec<_> = head.flatten().into_iter().collect();
                lenses.extend(render::lenses(&doc.uri, &views, insert, glyphs));
                json!(lenses)
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
                    render::hints(&views, &self.settings.indicators)
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

/// A local dependency: its path as written, and its `file://` URI when it exists beside the
/// manifest at `manifest`.
/// The page a dependency's lens opens: none for a manifest's own version, the source itself for
/// a fixed source, and otherwise the registry's package page.
fn page(dep: &Dep) -> String {
    match dep.registry {
        PROJECT => String::new(),
        SOURCE => source_page(dep).unwrap_or_else(|| registry::page(dep.registry, &dep.name)),
        _ => registry::page(dep.registry, &dep.name),
    }
}

/// The web address of a fixed source: its URL, without a `git+` prefix or a `#ref`, or the
/// repository a `github:owner/repo` shorthand names.
fn source_page(dep: &Dep) -> Option<String> {
    let req = dep.req.trim();
    let req = req.strip_prefix("git+").unwrap_or(req);
    let url = req.split('#').next().unwrap_or(req);
    if url.starts_with("https://") || url.starts_with("http://") {
        return Some(url.trim_end_matches(".git").to_owned());
    }
    let (host, path) = req.split_once(':')?;
    let host = match host {
        "github" => "github.com",
        "gitlab" => "gitlab.com",
        "bitbucket" => "bitbucket.org",
        _ => return None,
    };
    let path = path.split('#').next().unwrap_or(path);
    Some(format!("https://{host}/{path}"))
}

fn directory(manifest: &str, req: &str) -> State {
    let path = local_path(req).unwrap_or(req);
    let full = match path.strip_prefix("~/") {
        Some(rest) => std::env::home_dir().map(|home| home.join(rest)),
        // Cargo reads paths in `.cargo/config.toml` from the directory that holds `.cargo`.
        None => std::path::Path::new(manifest)
            .parent()
            .map(|dir| {
                if dir.ends_with(".cargo") {
                    dir.parent().unwrap_or(dir)
                } else {
                    dir
                }
            })
            .map(|dir| dir.join(path)),
    };
    let target = full
        .filter(|p| p.exists())
        .map(|p| file_uri(&p.to_string_lossy()));
    State::Directory {
        path: path.to_owned(),
        target,
    }
}
