//! Turns looked-up dependencies into what editors show: lenses, inlay hints, hovers, quick fixes,
//! and diagnostics. Pure functions of cached state, so every handler answers immediately.

use std::collections::HashMap;

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeLens, Command, Diagnostic,
    DiagnosticSeverity, Hover, HoverContents, InlayHint, InlayHintLabel, MarkupContent, MarkupKind,
    Position, Range, TextEdit, Uri, WorkspaceEdit,
};
use serde_json::json;

use crate::extract::Dep;
use crate::version::{Plan, Req, Version, level};

/// What is known about one dependency right now.
#[derive(Debug)]
pub(crate) enum State {
    Pending,
    Failed(String),
    /// The registry answered, but the requirement is not a range this server reads.
    Unreadable,
    Planned {
        plan: Plan,
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

/// The updates a view offers: label, version level, and replacement text.
pub(crate) fn picks(view: &View<'_>, insert: &str) -> Vec<(String, String)> {
    let State::Planned { plan, current } = &view.state else {
        return Vec::new();
    };
    plan.picks
        .iter()
        .map(|pick| {
            (
                format!("{} {}", level(current.as_ref(), pick), pick.raw),
                replacement(&view.dep.req, pick, insert),
            )
        })
        .filter(|(_, text)| *text != view.dep.req)
        .collect()
}

pub(crate) fn lenses(uri: &Uri, views: &[View<'_>], insert: &str) -> Vec<CodeLens> {
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
                "⚠ {} vulnerabilit{}",
                view.vulns.len(),
                if view.vulns.len() == 1 { "y" } else { "ies" }
            )));
        }
        match &view.state {
            State::Pending => out.push(open("⋯ checking".into())),
            State::Failed(error) => out.push(open(format!("✗ {}", short(error)))),
            State::Unreadable => out.push(open("? unreadable requirement".into())),
            State::Planned { plan, .. } => {
                let choices = picks(view, insert);
                if choices.is_empty() {
                    out.push(open(format!("✓ latest {}", plan.latest.raw)));
                }
                for (label, text) in choices {
                    out.push(CodeLens {
                        range: view.range,
                        command: Some(update_command(format!("↑ {label}"), uri, view, &text)),
                        data: None,
                    });
                }
            }
        }
    }
    out
}

fn short(error: &str) -> &str {
    let tail = error.rsplit(": ").next().unwrap_or(error);
    if tail.len() > 60 {
        &tail[..tail.floor_char_boundary(60)]
    } else {
        tail
    }
}

pub(crate) fn hints(views: &[View<'_>]) -> Vec<InlayHint> {
    views
        .iter()
        .filter_map(|view| {
            let label = match &view.state {
                State::Planned { plan, .. }
                    if plan.picks.is_empty() || plan.satisfied && view.dep.req.is_empty() =>
                {
                    "✓".to_owned()
                }
                State::Planned { plan, .. } => format!("→ {}", plan.latest.raw),
                State::Failed(_) => "✗".to_owned(),
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
        State::Planned { plan, current } => {
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

/// Whether a request limited to `only` kinds accepts [`ACTION_KIND`]; kinds are dot-separated
/// prefixes, so `refactor` accepts `refactor.rewrite`.
pub(crate) fn wants_actions(only: Option<&[CodeActionKind]>) -> bool {
    only.is_none_or(|only| {
        only.iter().any(|k| {
            ACTION_KIND
                .strip_prefix(k.as_str())
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

/// Every dependency moved to its latest version, as one edit.
pub(crate) fn update_all(uri: &Uri, views: &[View<'_>], insert: &str) -> Option<WorkspaceEdit> {
    let edits: Vec<TextEdit> = views
        .iter()
        .filter_map(|view| {
            picks(view, insert).pop().map(|(_, new_text)| TextEdit {
                range: view.range,
                new_text,
            })
        })
        .collect();
    (!edits.is_empty()).then(|| WorkspaceEdit {
        changes: Some(HashMap::from([(uri.clone(), edits)])),
        ..WorkspaceEdit::default()
    })
}

pub(crate) fn diagnostics(views: &[View<'_>]) -> Vec<Diagnostic> {
    views
        .iter()
        .filter(|view| !view.vulns.is_empty())
        .map(|view| Diagnostic {
            range: view.range,
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(lsp_types::NumberOrString::String("vulnerable".into())),
            source: Some("VersionLens".into()),
            message: format!(
                "{} {} has known vulnerabilities: {}",
                view.dep.name,
                view.dep.req,
                view.vulns.join(", ")
            ),
            ..Diagnostic::default()
        })
        .collect()
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
