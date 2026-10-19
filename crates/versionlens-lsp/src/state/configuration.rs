use std::io;
use std::path::Path;

use serde_json::Value;
use versionlens_engine::{workspace_exclusion_matches, workspace_exclusion_path};

use super::VersionLensLspState;

/// The settings sections read for each folder, in answer order.
pub(crate) const SECTIONS: [&str; 2] = ["versionlens", "files.exclude"];

const GENERATED_DIRECTORY_EXCLUSIONS: &[&str] = &[
    "**/.git/**",
    "**/.vscode/**",
    "**/bin/**",
    "**/bower_components/**",
    "**/build/**",
    "**/dist/**",
    "**/node_modules/**",
    "**/obj/**",
    "**/target/**",
    "**/vendor/**",
];

pub(crate) fn default_workspace_exclusions() -> Vec<String> {
    GENERATED_DIRECTORY_EXCLUSIONS
        .iter()
        .map(|pattern| (*pattern).to_owned())
        .collect()
}

/// The default exclusions plus the enabled patterns of the editor's
/// `files.exclude` section.
pub(crate) fn workspace_exclusions(files_exclude: &Value) -> Vec<String> {
    let mut exclusions = default_workspace_exclusions();
    let Some(configured) = files_exclude.as_object() else {
        return exclusions;
    };
    for (pattern, enabled) in configured {
        if enabled.as_bool() == Some(true) {
            expand_braces(pattern, &mut exclusions);
        }
    }
    exclusions.sort();
    exclusions.dedup();
    exclusions
}

fn expand_braces(pattern: &str, output: &mut Vec<String>) {
    let Some(open) = pattern.find('{') else {
        output.push(pattern.to_owned());
        return;
    };
    let Some(relative_close) = pattern[open + 1..].find('}') else {
        output.push(pattern.to_owned());
        return;
    };
    let close = open + 1 + relative_close;
    let choices = &pattern[open + 1..close];
    if !choices.contains(',') {
        output.push(pattern.to_owned());
        return;
    }
    for choice in choices.split(',') {
        let expanded = format!("{}{}{}", &pattern[..open], choice, &pattern[close + 1..]);
        expand_braces(&expanded, output);
    }
}

impl VersionLensLspState {
    #[cfg(test)]
    pub(crate) fn with_workspace_exclusions(self, exclusions: Vec<String>) -> Self {
        self.with_settings(super::FolderSettings {
            exclusions,
            ..super::FolderSettings::default()
        })
    }

    /// The `workspace/configuration` params: the settings sections the
    /// server reads, for each folder scope in the order of `scopes`.
    pub(crate) fn configuration_params(&self) -> Value {
        let scopes = self
            .scopes()
            .iter()
            .map(|root| Some(root.uri.to_string()))
            .collect::<Vec<_>>();
        let scopes = if scopes.is_empty() {
            vec![None]
        } else {
            scopes
        };
        let items = scopes
            .iter()
            .flat_map(|scope| {
                SECTIONS.iter().map(move |section| match scope {
                    Some(scope) => serde_json::json!({"scopeUri": scope, "section": section}),
                    None => serde_json::json!({"section": section}),
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({ "items": items })
    }

    pub(crate) fn watched_file_is_relevant(&self, uri: &str) -> bool {
        let Some(path) = versionlens_engine::workspace_path(uri) else {
            return false;
        };
        let Some((root, relative)) = self
            .scopes()
            .iter()
            .filter_map(|root| {
                let root = Path::new(root.path.as_deref()?);
                path.strip_prefix(root)
                    .ok()
                    .map(|relative| (root, relative))
            })
            .max_by_key(|(root, _)| root.components().count())
        else {
            return false;
        };
        let relative = workspace_exclusion_path(relative);
        let group = &self.groups()[self.group_of(uri)];
        !group
            .settings
            .exclusions
            .iter()
            .any(|pattern| workspace_exclusion_matches(pattern, &relative, false))
            && group
                .session
                .document_is_supported(&versionlens_model::DocumentInput::new(
                    uri.to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    versionlens_engine::workspace_file_uri(root),
                ))
    }

    /// Applies a folder change. New folders get the default settings until
    /// their settings are read.
    pub(crate) fn change_workspace_folders(
        &mut self,
        event: lsp_types::WorkspaceFoldersChangeEvent,
    ) -> io::Result<()> {
        if event.added.is_empty() && event.removed.is_empty() {
            return Ok(());
        }
        self.root_uri = None;
        self.workspace_folders
            .retain(|root| !event.removed.iter().any(|folder| folder.uri == root.uri));
        for folder in event.added {
            if !self
                .workspace_folders
                .iter()
                .any(|root| root.uri == folder.uri)
            {
                self.workspace_folders
                    .push(super::WorkspaceRoot::new(folder.uri));
            }
        }
        let roots = self
            .documents
            .keys()
            .map(|uri| (uri.clone(), self.workspace_root(uri)))
            .collect::<Vec<_>>();
        for (uri, root) in roots {
            if let Some(document) = self.documents.get_mut(&uri) {
                document.workspace_root = root;
            }
        }
        self.regroup()?;
        self.invalidate_workspace();
        self.synchronize_workspace_documents();
        Ok(())
    }

    /// Gives every document a new generation, so work started with the old
    /// settings is not used.
    pub(super) fn renew_generations(&mut self) -> io::Result<()> {
        let count = u64::try_from(self.revisions.len()).map_err(io::Error::other)?;
        let generation = self
            .generation
            .checked_add(count)
            .ok_or_else(|| io::Error::other("document generation exhausted"))?;
        for revision in self.revisions.values_mut() {
            self.generation += 1;
            revision.generation = self.generation;
        }
        self.generation = generation;
        Ok(())
    }

    /// Drops the cached registry data of every session. Every document gets
    /// a new generation, so no result computed from the old data is reused.
    pub(crate) fn clear_caches(&mut self) -> io::Result<()> {
        for group in self.groups() {
            group.session.try_clear_cache()?;
        }
        self.renew_generations()
    }

    pub(super) fn synchronize_workspace_documents(&mut self) {
        if self.modes.controller_managed {
            self.advance_document_generation();
            return;
        }
        let mut changed = false;
        for (index, group) in self.groups().iter().enumerate() {
            let documents = self
                .documents
                .keys()
                .filter(|uri| self.group_of(uri) == index)
                .filter_map(|uri| self.document_work(uri).map(|work| work.input))
                .collect();
            changed |= group.session.set_workspace_documents(documents);
        }
        if changed {
            self.advance_document_generation();
        }
    }

    pub(crate) fn invalidate_workspace(&mut self) {
        if !self.modes.controller_managed {
            for group in self.groups() {
                group.session.invalidate_workspace();
            }
        }
        self.advance_document_generation();
    }

    fn advance_document_generation(&mut self) {
        let Some(generation) = self.generation.checked_add(1) else {
            self.documents.clear();
            self.revisions.clear();
            self.cancel_pending_resolutions();
            return;
        };
        self.generation = generation;
        for revision in self.revisions.values_mut() {
            revision.generation = generation;
        }
    }

    pub(crate) fn document_uris(&self) -> Vec<String> {
        self.documents.keys().cloned().collect()
    }
}
