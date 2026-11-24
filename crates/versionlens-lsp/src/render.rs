//! Turns looked-up dependencies into what editors show: lenses, inlay hints, hovers, quick fixes,
//! and diagnostics. Pure functions of cached state, so every handler answers immediately.

use std::collections::HashMap;

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeLens, Command, Diagnostic,
    DiagnosticSeverity, Hover, HoverContents, InlayHint, InlayHintLabel, MarkupContent, MarkupKind,
    Position, Range, TextEdit, Uri, WorkspaceEdit,
};
use serde_json::json;

use crate::config::Indicators;
use crate::extract::Dep;
use crate::version::{self, Plan, Req, Version};

/// What is known about one dependency right now.
#[derive(Debug)]
pub(crate) enum State {
    Pending,
    Failed(String),
    /// The registry answered, but the requirement is not a range this server reads.
    Unreadable,
    Planned {
        plan: Box<Plan>,
        /// The requirement's floor, which update levels count from.
        current: Option<Version>,
    },
}

/// One dependency with its editor range, state, and known vulnerabilities.
#[derive(Debug)]
pub(crate) struct View<'a> {
    pub(crate) dep: &'a Dep,
    pub(crate) range: Range,
    pub(crate) state: State,
    pub(crate) vulns: &'a [String],
    pub(crate) page: String,
}

/// Byte offsets to LSP positions, which count UTF-16 code units.
pub(crate) struct Lines<'a> {
    text: &'a str,
    starts: Vec<usize>,
}

impl<'a> Lines<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        let starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        Self { text, starts }
    }

    pub(crate) fn position(&self, offset: usize) -> Position {
        let line = self
            .starts
            .partition_point(|s| *s <= offset)
            .saturating_sub(1);
        let start = self.starts.get(line).copied().unwrap_or(0);
        let column = self
            .text
            .get(start..offset)
            .map_or(0, |s| s.encode_utf16().count());
        Position::new(
            u32::try_from(line).unwrap_or(u32::MAX),
            u32::try_from(column).unwrap_or(u32::MAX),
        )
    }

    /// The byte offset of an LSP position, clamped to its line and to the text.
    pub(crate) fn offset(&self, at: Position) -> usize {
        let Some(&start) = self.starts.get(at.line as usize) else {
            return self.text.len();
        };
        let line = self.text[start..]
            .split_inclusive('\n')
            .next()
            .unwrap_or("");
        let mut units = 0;
        for (i, c) in line.char_indices() {
            if units >= at.character as usize || c == '\n' {
                return start + i;
            }
            units += c.len_utf16();
        }
        start + line.len()
    }

    pub(crate) fn range(&self, span: &std::ops::Range<usize>) -> Range {
        Range::new(self.position(span.start), self.position(span.end))
    }
}

/// The requirement text that moves `req` to `pick` in the user's own style: the operator stays,
/// ranges collapse to their leading operator, and a bare name gets the file type's prefix.
pub(crate) fn replacement(req: &str, pick: &Version, insert: &str) -> String {
    if req.is_empty() {
        return format!("{insert}{}", pick.raw);
    }
    if req.starts_with(['[', '(']) {
        return format!("[{}]", pick.raw);
    }
    let first = req
        .split([',', '|'])
        .next()
        .unwrap_or(req)
        .split(" - ")
        .next()
        .unwrap_or(req)
        .trim();
    let at = first.find(|c: char| c.is_ascii_digit()).map_or(0, |i| {
        if i > 0 && first[..i].ends_with(['v', 'V']) {
            i - 1
        } else {
            i
        }
    });
    let (operator, like) = first.split_at(at);
    let like = like.split_whitespace().next().unwrap_or(like);
    format!("{operator}{}", pick.render(like, false))
}

fn edit(uri: &Uri, range: Range, text: String) -> WorkspaceEdit {
    WorkspaceEdit {
        changes: Some(HashMap::from([(
            uri.clone(),
            vec![TextEdit {
                range,
                new_text: text,
            }],
        )])),
        ..WorkspaceEdit::default()
    }
}

fn update_command(title: String, uri: &Uri, view: &View<'_>, text: &str) -> Command {
    let args = vec![json!(uri), json!(view.range), json!(text)];
    Command {
        title,
        command: "versionlens.update".into(),
        arguments: Some(args),
    }
}

/// The updates a view offers, newest first: the 1.x label with its version, and the
/// replacement text.
pub(crate) fn picks(view: &View<'_>, insert: &str) -> Vec<(String, String)> {
    let State::Planned { plan, .. } = &view.state else {
        return Vec::new();
    };
    plan.picks
        .iter()
        .map(|(label, pick)| {
            (
                format!("{label} {}", pick.raw),
                replacement(&view.dep.req, pick, insert),
            )
        })
        .filter(|(_, text)| *text != view.dep.req)
        .collect()
}

pub(crate) fn lenses(
    uri: &Uri,
    views: &[View<'_>],
    insert: &str,
    glyphs: &Indicators,
) -> Vec<CodeLens> {
    let mut out = Vec::new();
    for view in views {
        let lens = |title: String, command: &str, args: Vec<serde_json::Value>| CodeLens {
            range: view.range,
            command: Some(Command {
                title,
                command: command.into(),
                arguments: Some(args),
            }),
            data: None,
        };
        let open = |title: String| lens(title, "versionlens.open", vec![json!(view.page)]);
        if !view.vulns.is_empty() {
            out.push(open(format!(
                "{} {} vulnerabilit{}",
                glyphs.vulnerable,
                view.vulns.len(),
                if view.vulns.len() == 1 { "y" } else { "ies" }
            )));
        }
        match &view.state {
            State::Pending => out.push(open(format!("{} checking", glyphs.checking))),
            State::Failed(error) => out.push(open(format!("{} {}", glyphs.failed, short(error)))),
            State::Unreadable => out.push(open(format!(
                "{} unreadable requirement",
                glyphs.unreadable
            ))),
            State::Planned { plan, .. } => {
                let choices = picks(view, insert);
                out.push(open(status(plan, choices.is_empty(), glyphs)));
                for (label, text) in choices {
                    out.push(CodeLens {
                        range: view.range,
                        command: Some(update_command(
                            format!("{} {label}", glyphs.update),
                            uri,
                            view,
                            &text,
                        )),
                        data: None,
                    });
                }
            }
        }
    }
    out
}

/// The status lens that leads a dependency's updates, as the original 1.x drew it: what the
/// requirement resolves to now, against the latest version.
fn status(plan: &Plan, current: bool, glyphs: &Indicators) -> String {
    match &plan.resolved {
        None => format!("{} no match", glyphs.no_match),
        // Already on the latest, with nothing left to move.
        Some(v) if *v >= plan.latest && current => format!("{} latest {}", glyphs.latest, v.raw),
        // Resolves to the latest, though the requirement still admits older versions.
        Some(v) if *v >= plan.latest => format!("{} satisfies latest {}", glyphs.latest, v.raw),
        Some(v) => format!("{} satisfies {}", glyphs.satisfies, v.raw),
    }
}

fn short(error: &str) -> &str {
    let tail = error.rsplit(": ").next().unwrap_or(error);
    if tail.len() > 60 {
        &tail[..tail.floor_char_boundary(60)]
    } else {
        tail
    }
}

pub(crate) fn hints(views: &[View<'_>], glyphs: &Indicators) -> Vec<InlayHint> {
    views
        .iter()
        .filter_map(|view| {
            let label = match &view.state {
                State::Planned { plan, .. }
                    if plan.picks.is_empty() || plan.satisfied && view.dep.req.is_empty() =>
                {
                    glyphs.latest.clone()
                }
                State::Planned { plan, .. } => format!("{} {}", glyphs.newer, plan.latest.raw),
                State::Failed(_) => glyphs.failed.clone(),
                State::Pending | State::Unreadable => return None,
            };
            Some(InlayHint {
                position: view.range.end,
                label: InlayHintLabel::String(label),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: Some(true),
                padding_right: None,
                data: None,
            })
        })
        .collect()
}

pub(crate) fn hover(view: &View<'_>) -> Hover {
    let dep = view.dep;
    let status = match &view.state {
        State::Pending => "Checking the registry…".to_owned(),
        State::Failed(error) => format!("Lookup failed: `{error}`"),
        State::Unreadable => format!("`{}` is not a version range VersionLens reads.", dep.req),
        State::Planned { plan, current, .. } => {
            let current = current.as_ref().map_or("none", |v| v.raw.as_str());
            format!(
                "Requires `{}` · from `{current}` · latest `{}`",
                dep.req, plan.latest.raw
            )
        }
    };
    let vulns = view.vulns.iter().fold(String::new(), |text, id| {
        text + "\n\n⚠ [" + id + "](https://osv.dev/vulnerability/" + id + ")"
    });
    let text = format!(
        "**{}** · {} · [registry page]({})\n\n{status}{vulns}",
        dep.name, dep.registry, view.page
    );
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: text,
        }),
        range: Some(view.range),
    }
}

/// Version updates rewrite a requirement rather than fix a diagnostic. The LSP client of IDEA
/// and the other `IntelliJ` IDEs hides `quickfix` actions that come without a diagnostic.
pub(crate) const ACTION_KIND: &str = "refactor.rewrite";

/// Whether a request limited to `only` kinds accepts `kind`; kinds are dot-separated prefixes,
/// so `refactor` accepts `refactor.rewrite`.
pub(crate) fn wants(kind: &str, only: Option<&[CodeActionKind]>) -> bool {
    only.is_none_or(|only| {
        only.iter().any(|k| {
            kind.strip_prefix(k.as_str())
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
        })
    })
}

pub(crate) fn actions(uri: &Uri, views: &[View<'_>], insert: &str) -> Vec<CodeActionOrCommand> {
    let mut out = Vec::new();
    for view in views {
        for (index, (label, text)) in picks(view, insert).into_iter().enumerate() {
            out.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Update {} to {label}", view.dep.name),
                kind: Some(CodeActionKind::from(ACTION_KIND)),
                edit: Some(edit(uri, view.range, text)),
                is_preferred: Some(index == 0),
                ..CodeAction::default()
            }));
        }
    }
    out
}

/// The bulk update levels: `latest` moves every dependency to its newest version, and `major`,
/// `minor`, and `patch` move only the dependencies whose update of that size exists.
pub(crate) const LEVELS: &[&str] = &["latest", "major", "minor", "patch"];

/// The edits that move each dependency by `level`, one of [`LEVELS`].
pub(crate) fn updates(views: &[View<'_>], insert: &str, level: &str) -> Vec<TextEdit> {
    views
        .iter()
        .filter_map(|view| {
            let State::Planned { plan, current } = &view.state else {
                return None;
            };
            // Steps run oldest first and hold one version per level counted from the floor.
            let pick = if level == "latest" {
                plan.steps.last()
            } else {
                plan.steps
                    .iter()
                    .find(|v| version::level(current.as_ref(), v) == level)
            }?;
            let new_text = replacement(&view.dep.req, pick, insert);
            (new_text != view.dep.req).then_some(TextEdit {
                range: view.range,
                new_text,
            })
        })
        .collect()
}

/// Vulnerability warnings, and with `updates` a diagnostic of that severity for each dependency
/// that has a newer version.
pub(crate) fn diagnostics(
    views: &[View<'_>],
    insert: &str,
    updates: Option<DiagnosticSeverity>,
) -> Vec<Diagnostic> {
    let diagnostic = |view: &View<'_>, severity, code: &str, message| Diagnostic {
        range: view.range,
        severity: Some(severity),
        code: Some(lsp_types::NumberOrString::String(code.into())),
        source: Some("VersionLens".into()),
        message,
        ..Diagnostic::default()
    };
    let mut out = Vec::new();
    for view in views {
        if !view.vulns.is_empty() {
            let message = format!(
                "{} {} has known vulnerabilities: {}",
                view.dep.name,
                view.dep.req,
                view.vulns.join(", ")
            );
            out.push(diagnostic(
                view,
                DiagnosticSeverity::WARNING,
                "vulnerable",
                message,
            ));
        }
        if let Some(severity) = updates
            && let Some((label, _)) = picks(view, insert).into_iter().next()
        {
            let message = format!("{} can update to {label}", view.dep.name);
            out.push(diagnostic(view, severity, "outdated", message));
        }
    }
    out
}

/// The summary lens that heads a manifest: dependency, update, failure, and vulnerability
/// counts; it refreshes the manifest's versions.
pub(crate) fn stats(
    uri: &Uri,
    views: &[View<'_>],
    insert: &str,
    glyphs: &Indicators,
) -> Option<CodeLens> {
    if views.is_empty() {
        return None;
    }
    let count = |test: &dyn Fn(&View<'_>) -> bool| views.iter().filter(|v| test(v)).count();
    let outdated = count(&|v| !picks(v, insert).is_empty());
    let failed = count(&|v| matches!(v.state, State::Failed(_)));
    let vulnerable = count(&|v| !v.vulns.is_empty());
    let pending = count(&|v| matches!(v.state, State::Pending));
    let mut parts = vec![format!("{} dependencies", views.len())];
    for (n, glyph, text) in [
        (outdated, &glyphs.update, "outdated"),
        (failed, &glyphs.failed, "failed"),
        (vulnerable, &glyphs.vulnerable, "vulnerable"),
        (pending, &glyphs.checking, "checking"),
    ] {
        if n > 0 {
            parts.push(format!("{glyph} {n} {text}"));
        }
    }
    Some(CodeLens {
        range: Range::default(),
        command: Some(Command {
            title: parts.join(" · "),
            command: "versionlens.refresh".into(),
            arguments: Some(vec![json!(uri)]),
        }),
        data: None,
    })
}

/// The version a requirement currently resolves to, for OSV and update levels: the newest listed
/// version it allows, or else its floor.
pub(crate) fn current(req: &Req, versions: &[Version]) -> Option<Version> {
    versions
        .iter()
        .filter(|v| req.matches(v))
        .max()
        .cloned()
        .or_else(|| req.floor().cloned())
}

#[cfg(test)]
mod tests;
