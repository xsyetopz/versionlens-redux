//! What each open manifest offers right now: the `versionlens/status` notification that editors
//! turn into toolbar icons and status text, and the same commands as source code actions for
//! editors whose extensions cannot add menus.

use lsp_types::{CodeAction, CodeActionKind, CodeActionOrCommand, Command};
use serde_json::{Value, json};

use super::{Doc, Server, key};
use crate::render::{self, LEVELS, State};
use crate::sort;

/// The kind of the command actions, under `source` so editors list them with source actions.
pub(super) const SOURCE_KIND: &str = "source.versionlens";

impl Server {
    /// The install command configured for a manifest: by file type id, then by registry id.
    pub(super) fn install_command(&self, doc: &Doc) -> Option<&str> {
        let install = &self.settings.install;
        install
            .get(doc.file.id)
            .or_else(|| install.get(doc.file.registry))
            .map(String::as_str)
    }

    pub(super) fn sortable(doc: &Doc) -> bool {
        let commas = matches!(doc.file.format, crate::flatten::Format::Json);
        !sort::runs(&doc.text, &doc.deps, commas).is_empty()
    }

    fn status_of(&self, doc: &Doc) -> Value {
        let views = self.views(doc);
        let failed = views
            .iter()
            .filter(|v| matches!(v.state, State::Failed(_)))
            .count();
        let vulnerable = views.iter().filter(|v| !v.vulns.is_empty()).count();
        let uri = doc.uri.as_str();
        json!({
            "uri": uri,
            "enabled": self.settings.enabled,
            "prereleases": self.settings.prereleases,
            "busy": self.installing.contains(uri) || doc.deps.iter().any(|d| self.inflight.contains(&key(d))),
            "dependencies": doc.deps.len(),
            "outdated": render::updates(&views, doc.file.insert, "latest").len(),
            "failed": failed,
            "vulnerable": vulnerable,
            "sortable": Self::sortable(doc),
            "install": self.install_command(doc),
        })
    }

    /// Sends `versionlens/status` for each open manifest whose status changed.
    pub(super) fn send_status(&mut self) {
        let changed: Vec<(String, Value)> = self
            .docs
            .values()
            .filter(|d| d.open)
            .map(|d| (d.uri.as_str().to_owned(), self.status_of(d)))
            .filter(|(uri, status)| self.status.get(uri) != Some(status))
            .collect();
        for (uri, status) in changed {
            self.notify("versionlens/status", status.clone());
            self.status.insert(uri, status);
        }
    }

    /// Every command that applies to the manifest, as code actions.
    pub(super) fn source_actions(&self, doc: &Doc) -> Vec<CodeActionOrCommand> {
        let uri = json!(doc.uri);
        let mut list: Vec<(String, &str, Vec<Value>)> = Vec::new();
        if self.settings.enabled {
            let views = self.views(doc);
            for level in LEVELS {
                let count = render::updates(&views, doc.file.insert, level).len();
                if count > 0 {
                    list.push((
                        format!(
                            "Update {count} dependenc{} to {level}",
                            if count == 1 { "y" } else { "ies" }
                        ),
                        "versionlens.updateAll",
                        vec![uri.clone(), json!(level)],
                    ));
                }
            }
        }
        for level in LEVELS {
            list.push((
                format!("Update workspace dependencies to {level}"),
                "versionlens.updateWorkspace",
                vec![json!(level)],
            ));
        }
        if Self::sortable(doc) {
            list.push((
                "Sort dependencies".into(),
                "versionlens.sort",
                vec![uri.clone()],
            ));
        }
        if let Some(command) = self.install_command(doc) {
            list.push((
                format!("Install dependencies ({command})"),
                "versionlens.install",
                vec![uri.clone()],
            ));
        }
        let (shown, pre) = (self.settings.enabled, self.settings.prereleases);
        let (versions, show) = if shown {
            ("Hide versions", "versionlens.hide")
        } else {
            ("Show versions", "versionlens.show")
        };
        let (prereleases, toggle) = if pre {
            ("Hide prereleases", "versionlens.hidePrereleases")
        } else {
            ("Show prereleases", "versionlens.showPrereleases")
        };
        list.extend([
            (versions.into(), show, vec![uri.clone()]),
            (prereleases.into(), toggle, vec![uri.clone()]),
            (
                "Refresh versions".into(),
                "versionlens.refresh",
                vec![uri.clone()],
            ),
            ("Clear the cache".into(), "versionlens.clearCache", vec![]),
        ]);
        if self.caps.auth_actions {
            list.extend([
                (
                    "Add URL authentication".into(),
                    "versionlens.addAuthentication",
                    vec![uri.clone()],
                ),
                (
                    "Remove URL authentication".into(),
                    "versionlens.removeAuthentication",
                    vec![],
                ),
            ]);
        }
        list.into_iter()
            .map(|(title, command, args)| {
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("VersionLens: {title}"),
                    kind: Some(CodeActionKind::from(SOURCE_KIND)),
                    command: Some(Command {
                        title,
                        command: command.into(),
                        arguments: Some(args),
                    }),
                    ..CodeAction::default()
                })
            })
            .collect()
    }
}
