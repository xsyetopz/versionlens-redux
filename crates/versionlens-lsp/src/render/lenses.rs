//! Code lenses: the status of each dependency and the updates it offers, as the original 1.x
//! drew them above the line.

use lsp_types::{CodeLens, Command, Uri};
use serde_json::json;

use super::{State, View, picks, replacement};
use crate::config::Indicators;
use crate::git_ref::{self, Ref};
use crate::version::Plan;

fn update_command(title: String, uri: &Uri, view: &View<'_>, text: &str) -> Command {
    let args = vec![json!(uri), json!(view.range), json!(text)];
    Command {
        title,
        command: "versionlens.update".into(),
        arguments: Some(args),
    }
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
            State::Directory {
                path,
                target: Some(target),
            } => out.push(lens(
                format!("{} file://{path}", glyphs.directory),
                "versionlens.open",
                vec![json!(target)],
            )),
            State::Directory { path, target: None } => {
                out.push(open(format!("{} not found {path}", glyphs.failed)));
            }
            State::Source(label) => out.push(open(label.clone())),
            State::Commit(found) => {
                out.push(open(commit_status(found, &view.dep.req, glyphs)));
                out.extend(
                    picks(view, insert)
                        .into_iter()
                        .map(|(label, text)| CodeLens {
                            range: view.range,
                            command: Some(update_command(
                                format!("{} {label}", glyphs.update),
                                uri,
                                view,
                                &text,
                            )),
                            data: None,
                        }),
                );
            }
            State::Own(next) => out.extend(next.iter().map(|(label, text)| CodeLens {
                range: view.range,
                command: Some(update_command(
                    format!("{} {label}", glyphs.update),
                    uri,
                    view,
                    text,
                )),
                data: None,
            })),
            State::Planned { plan, .. } => {
                let choices = picks(view, insert);
                out.push(open(status(plan, choices.is_empty(), glyphs)));
                if let Some(command) = build_command(uri, view, insert, glyphs) {
                    out.push(CodeLens {
                        range: view.range,
                        command: Some(command),
                        data: None,
                    });
                }
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
pub(super) fn status(plan: &Plan, current: bool, glyphs: &Indicators) -> String {
    if let Some(v) = &plan.yanked {
        return format!("{} yanked {}", glyphs.yanked, v.raw);
    }
    match &plan.resolved {
        Some(v) if plan.deprecated => format!("{} deprecated {}", glyphs.deprecated, v.raw),
        None => format!("{} no match", glyphs.no_match),
        // Already on the latest, with nothing left to move.
        Some(v) if *v >= plan.latest && current => format!("{} latest {}", glyphs.latest, v.raw),
        // Resolves to the latest, though the requirement still admits older versions.
        Some(v) if *v >= plan.latest => format!("{} satisfies latest {}", glyphs.latest, v.raw),
        Some(v) => format!("{} satisfies {}", glyphs.satisfies, v.raw),
    }
}

/// The status lens of a pinned reference: the head a branch runs, the commit a tag names, and
/// whether a pinned commit is its branch's head or lags behind it.
fn commit_status(found: &Ref, pin: &str, glyphs: &Indicators) -> String {
    match found {
        Ref::Branch { name, head } => format!("branch {name} head {}", git_ref::short(head)),
        Ref::Tag { name, commit } => format!("tag {name} at {}", git_ref::short(commit)),
        Ref::Pinned { branch, head, tag } => {
            let tag = tag
                .as_ref()
                .map_or_else(String::new, |t| format!("tag {t} · "));
            if found.lags(pin) {
                format!(
                    "{} {tag}behind {branch} head {}",
                    glyphs.satisfies,
                    git_ref::short(head)
                )
            } else {
                format!("{} {tag}{branch} head", glyphs.latest)
            }
        }
    }
}

/// The original's `change build` lens: it asks which build of the resolved version to switch to.
pub(super) fn build_command(
    uri: &Uri,
    view: &View<'_>,
    insert: &str,
    glyphs: &Indicators,
) -> Option<Command> {
    let State::Planned { plan, .. } = &view.state else {
        return None;
    };
    let mut choices: Vec<String> = Vec::new();
    for build in &plan.builds {
        let Some(text) = replacement(&view.dep.req, build, insert, view.dep.precise) else {
            continue;
        };
        if text != view.dep.req && !choices.contains(&text) {
            choices.push(text);
        }
    }
    (!choices.is_empty()).then(|| Command {
        title: format!("{} change build", glyphs.build),
        command: "versionlens.build".into(),
        arguments: Some(vec![
            json!(uri),
            json!(view.range),
            json!(view.dep.name),
            json!(choices),
        ]),
    })
}

pub(super) fn short(error: &str) -> &str {
    let tail = error.rsplit(": ").next().unwrap_or(error);
    if tail.len() > 60 {
        &tail[..tail.floor_char_boundary(60)]
    } else {
        tail
    }
}
