use std::io;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;
use versionlens_core::{
    SessionConfig, SessionConfigInput, file_pattern_config_from_name, version_lens_session,
    workspace_exclusion_matches, workspace_exclusion_path,
};

use super::VersionLensLspState;

const GENERATED_DIRECTORY_EXCLUSIONS: &[&str] = &[
    "**/.git/**",
    "**/.vscode/**",
    "**/bin/**",
    "**/bower_components/**",
    "**/build/**",
    "**/dist/**",
    "**/node_modules/**",
    "**/target/**",
    "**/vendor/**",
];

pub(super) fn default_workspace_exclusions() -> Vec<String> {
    GENERATED_DIRECTORY_EXCLUSIONS
        .iter()
        .map(|pattern| (*pattern).to_owned())
        .collect()
}

pub(crate) fn workspace_exclusions(settings: &Value) -> Vec<String> {
    let mut exclusions = default_workspace_exclusions();
    let Some(configured) = settings
        .get("files")
        .and_then(|files| files.get("exclude"))
        .and_then(Value::as_object)
    else {
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

pub(crate) fn session_configuration(value: Value) -> Result<SessionConfig, String> {
    if !value.is_object() && !value.is_null() {
        return Err("expected a settings object".to_owned());
    }
    let mut value = value;
    let cache_duration_minutes = take_setting::<f64>(&mut value, "cacheDurationMinutes")?;
    let cache_ttl_seconds = take_setting::<u32>(&mut value, "cacheTtlSeconds")?;
    let enabled_providers = value
        .as_object_mut()
        .and_then(|settings| settings.remove("enabledProviders"))
        .map(serde_json::from_value::<Vec<String>>)
        .transpose()
        .map_err(|error| error.to_string())?;
    let file_patterns = value
        .get_mut("providers")
        .and_then(Value::as_object_mut)
        .and_then(|providers| providers.remove("filePatterns"))
        .map(serde_json::from_value::<Vec<FilePatternInput>>)
        .transpose()
        .map_err(|error| error.to_string())?;
    let defaults: SessionConfig = SessionConfigInput {
        cache_duration_minutes,
        cache_ttl_seconds,
        enabled_providers,
        ..SessionConfigInput::default()
    }
    .into();
    let mut configuration = serde_json::to_value(defaults).map_err(|error| error.to_string())?;
    if !value.is_null() {
        merge(&mut configuration, value);
    }
    let mut configuration = serde_json::from_value::<SessionConfig>(configuration)
        .map_err(|error| error.to_string())?;
    if let Some(file_patterns) = file_patterns {
        configuration.providers.file_patterns = file_patterns
            .into_iter()
            .filter_map(|pattern| {
                file_pattern_config_from_name(&pattern.ecosystem, pattern.pattern)
            })
            .collect();
    }
    Ok(configuration)
}

fn take_setting<T>(value: &mut Value, key: &str) -> Result<Option<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    value
        .as_object_mut()
        .and_then(|settings| settings.remove(key))
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| error.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilePatternInput {
    ecosystem: String,
    pattern: String,
}

fn merge(target: &mut Value, source: Value) {
    match (target, source) {
        (Value::Object(target), Value::Object(source)) => {
            for (key, value) in source {
                merge(target.entry(key).or_insert(Value::Null), value);
            }
        }
        (target, source) => *target = source,
    }
}

impl VersionLensLspState {
    pub(crate) fn with_workspace_exclusions(mut self, exclusions: Vec<String>) -> Self {
        self.exclusions = exclusions;
        self
    }

    pub(crate) fn replace_workspace_exclusions(&mut self, exclusions: Vec<String>) -> bool {
        if self.exclusions == exclusions {
            return false;
        }
        self.exclusions = exclusions;
        self.invalidate_workspace();
        true
    }

    pub(crate) fn configuration_is(&self, config: &SessionConfig) -> bool {
        &self.configuration == config
    }

    pub(crate) fn watched_file_is_relevant(&self, uri: &str) -> bool {
        let Some(path) = versionlens_core::workspace_path(uri) else {
            return false;
        };
        let roots = if self.workspace_folders.is_empty() {
            self.root_uri.iter().collect::<Vec<_>>()
        } else {
            self.workspace_folders.iter().collect::<Vec<_>>()
        };
        let Some((root, relative)) = roots
            .into_iter()
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
        !self
            .exclusions
            .iter()
            .any(|pattern| workspace_exclusion_matches(pattern, &relative, false))
            && self
                .session
                .document_is_supported(&versionlens_model::DocumentInput::new(
                    uri.to_owned(),
                    "".to_owned(),
                    "".to_owned(),
                    versionlens_core::workspace_file_uri(root),
                ))
    }

    pub(crate) fn change_workspace_folders(
        &mut self,
        event: lsp_types::WorkspaceFoldersChangeEvent,
    ) {
        if event.added.is_empty() && event.removed.is_empty() {
            return;
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
        self.invalidate_workspace();
        self.synchronize_workspace_documents();
    }

    pub(crate) fn replace_configuration(&mut self, config: SessionConfig) -> io::Result<()> {
        let session = version_lens_session(config.clone());
        let session = if self.modes.persistent {
            session.with_application_cache()?
        } else {
            session
        };
        let count = u64::try_from(self.revisions.len()).map_err(io::Error::other)?;
        let generation = self
            .generation
            .checked_add(count)
            .ok_or_else(|| io::Error::other("document generation exhausted"))?;
        self.session.cancel_pending_resolutions();
        self.session = session;
        self.configuration = config;
        for revision in self.revisions.values_mut() {
            self.generation += 1;
            revision.generation = self.generation;
        }
        self.generation = generation;
        self.synchronize_workspace_documents();
        Ok(())
    }

    pub(super) fn synchronize_workspace_documents(&mut self) {
        if self.modes.controller_managed {
            self.advance_document_generation();
            return;
        }
        let documents = self
            .documents
            .keys()
            .filter_map(|uri| self.document_work(uri).map(|work| work.input))
            .collect();
        if !self.session.set_workspace_documents(documents) {
            return;
        }
        self.advance_document_generation();
    }

    pub(crate) fn invalidate_workspace(&mut self) {
        if !self.modes.controller_managed {
            self.session.invalidate_workspace();
        }
        self.advance_document_generation();
    }

    fn advance_document_generation(&mut self) {
        let Some(generation) = self.generation.checked_add(1) else {
            self.documents.clear();
            self.revisions.clear();
            self.session.cancel_pending_resolutions();
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
