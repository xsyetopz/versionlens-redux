use std::collections::HashMap;
use std::path::Path;

use lsp_types::{
    CodeLens, CodeLensOptions, Diagnostic, ExecuteCommandOptions, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
    WorkspaceFolder,
};
use serde::{Deserialize, Serialize};
use versionlens_core::{WorkspaceCheckingOptions, WorkspaceDiscoveryOptions};
use versionlens_model::DocumentInput;

/// Updates one dependency: `{ uri, dependency, level?, version? }`.
pub(crate) const UPDATE_COMMAND: &str = "versionlens.server.update";
/// Updates every dependency of a document at a level: `{ uri, level? }`.
pub(crate) const UPDATE_ALL_COMMAND: &str = "versionlens.server.updateAll";
/// Sorts the dependencies of a document: `{ uri }`.
pub(crate) const SORT_COMMAND: &str = "versionlens.server.sort";
/// Sets a dependency to one build: `{ uri, dependency, build }`.
pub(crate) const CHOOSE_BUILD_COMMAND: &str = "versionlens.server.chooseBuild";
/// Drops cached registry data and checks the workspace again: `{}`.
pub(crate) const CLEAR_CACHE_COMMAND: &str = "versionlens.server.clearCache";
/// Describes the manifest of a document: `{ uri }`.
pub(crate) const MANIFEST_INFO_COMMAND: &str = "versionlens.server.manifestInfo";

pub(crate) const DISPLAY_CODE_LENS_COMMAND: &str = "versionlens.displayCodeLens";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionLensTextDocument {
    pub uri: String,
    pub language_id: String,
    pub text: String,
    pub workspace_root: Option<String>,
}

#[derive(Debug)]
struct WorkspaceRoot {
    uri: Uri,
    path: Option<String>,
    input_root: Option<String>,
    settings: FolderSettings,
}

impl WorkspaceRoot {
    fn new(uri: Uri) -> Self {
        let normalized = normalized_file_path(uri.as_str()).map(|path| {
            if path.is_file() {
                path.parent()
                    .map_or_else(|| path.clone(), Path::to_path_buf)
            } else {
                path
            }
        });
        let input_root = normalized
            .as_deref()
            .and_then(versionlens_core::workspace_file_uri);
        let path = normalized.and_then(|path| path.into_os_string().into_string().ok());
        Self {
            uri,
            path,
            input_root,
            settings: FolderSettings::default(),
        }
    }

    fn contains(&self, document_uri: &Uri) -> bool {
        if self.uri.scheme().map(|scheme| scheme.as_str())
            != document_uri.scheme().map(|scheme| scheme.as_str())
            || self.uri.authority().map(|authority| authority.as_str())
                != document_uri.authority().map(|authority| authority.as_str())
        {
            return false;
        }
        if let (Some(root), Some(document)) = (
            self.path.as_deref(),
            normalized_file_path(document_uri.as_str()),
        ) {
            return document.starts_with(root);
        }
        let root = self.uri.path().as_str().trim_end_matches('/');
        let document = document_uri.path().as_str();
        document == root
            || document
                .strip_prefix(root)
                .is_some_and(|relative| relative.starts_with('/'))
    }
}

fn normalized_file_path(uri: &str) -> Option<std::path::PathBuf> {
    let path = versionlens_core::workspace_path(uri)?;
    Some(path.canonicalize().unwrap_or_else(|_| {
        path.parent()
            .and_then(|parent| parent.canonicalize().ok())
            .zip(path.file_name())
            .map_or_else(|| path.clone(), |(parent, name)| parent.join(name))
    }))
}

#[derive(Debug)]
pub struct VersionLensLspState {
    groups: Vec<SessionGroup>,
    /// The settings when there is no workspace folder.
    unscoped_settings: FolderSettings,
    documents: HashMap<String, VersionLensTextDocument>,
    revisions: HashMap<String, DocumentRevision>,
    generation: u64,
    root_uri: Option<WorkspaceRoot>,
    workspace_folders: Vec<WorkspaceRoot>,
    pub(crate) client: ClientSupport,
    modes: StateModes,
}

#[derive(Debug, Default)]
struct StateModes {
    /// Where sessions store results across restarts. `None` keeps them in
    /// memory only.
    cache_directory: Option<std::path::PathBuf>,
    controller_managed: bool,
}

#[derive(Debug, Default)]
pub(crate) struct ClientSupport {
    pub(crate) workspace_edits: bool,
    /// The client takes versioned document changes in a workspace edit.
    pub(crate) document_changes: bool,
    pub(crate) code_lens_refresh: bool,
    pub(crate) watched_files_dynamic: bool,
    /// The client answers `workspace/configuration`.
    pub(crate) configuration: bool,
}

#[derive(Debug, Clone)]
struct DocumentRevision {
    generation: u64,
    version: Option<i32>,
    path: Option<std::path::PathBuf>,
    core_uri: String,
}

#[derive(Clone)]
pub(crate) struct DocumentWork {
    pub(crate) uri: String,
    pub(crate) input: DocumentInput,
    pub(crate) generation: u64,
    pub(crate) version: Option<i32>,
}

#[derive(Clone)]
pub(crate) struct ResolvedDocument {
    pub(crate) code_lenses: Vec<CodeLens>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

pub(crate) struct CheckedDocument {
    pub(crate) uri: String,
    pub(crate) version: Option<i32>,
    pub(crate) input: DocumentInput,
    pub(crate) open: bool,
}

impl VersionLensLspState {
    /// Gives every folder the same settings.
    #[cfg(test)]
    pub(crate) fn with_settings(mut self, settings: FolderSettings) -> Self {
        let count = self.scopes().len().max(1);
        self.replace_settings(vec![settings; count])
            .expect("in-memory sessions do not fail");
        self
    }

    #[cfg(test)]
    pub(crate) fn with_config(self, config: versionlens_core::SessionConfig) -> Self {
        self.with_settings(FolderSettings {
            config,
            ..FolderSettings::default()
        })
    }

    pub(crate) fn with_client(mut self, capabilities: lsp_types::ClientCapabilities) -> Self {
        if let Some(workspace) = capabilities.workspace {
            self.client.document_changes = workspace
                .workspace_edit
                .is_some_and(|edit| edit.document_changes == Some(true));
            self.client.workspace_edits =
                workspace.apply_edit == Some(true) && self.client.document_changes;
            self.client.code_lens_refresh = workspace
                .code_lens
                .is_some_and(|lens| lens.refresh_support == Some(true));
            self.client.watched_files_dynamic = workspace
                .did_change_watched_files
                .is_some_and(|files| files.dynamic_registration == Some(true));
            self.client.configuration = workspace.configuration == Some(true);
        }
        self
    }

    pub fn standard() -> Self {
        Self::with_workspace(None, Vec::new())
    }

    pub(crate) fn with_workspace(
        root_uri: Option<Uri>,
        workspace_folders: Vec<WorkspaceFolder>,
    ) -> Self {
        let root_uri = root_uri.map(WorkspaceRoot::new);
        let workspace_folders = workspace_folders
            .into_iter()
            .map(|folder| WorkspaceRoot::new(folder.uri))
            .collect::<Vec<_>>();
        let scope_count = if workspace_folders.is_empty() {
            usize::from(root_uri.is_some())
        } else {
            workspace_folders.len()
        };
        Self {
            groups: vec![SessionGroup::initial(scope_count.max(1))],
            unscoped_settings: FolderSettings::default(),
            documents: HashMap::new(),
            revisions: HashMap::new(),
            generation: 0,
            client: ClientSupport::default(),
            modes: StateModes::default(),
            root_uri,
            workspace_folders,
        }
    }

    pub(crate) fn use_workspace_controller(&mut self) {
        self.modes.controller_managed = true;
    }

    pub fn server_capabilities() -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    ..TextDocumentSyncOptions::default()
                },
            )),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false),
            }),
            code_action_provider: Some(lsp_types::CodeActionProviderCapability::Options(
                lsp_types::CodeActionOptions {
                    code_action_kinds: Some(vec![lsp_types::CodeActionKind::QUICKFIX]),
                    resolve_provider: Some(false),
                    ..lsp_types::CodeActionOptions::default()
                },
            )),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: [
                    DISPLAY_CODE_LENS_COMMAND,
                    UPDATE_COMMAND,
                    UPDATE_ALL_COMMAND,
                    SORT_COMMAND,
                    CHOOSE_BUILD_COMMAND,
                    CLEAR_CACHE_COMMAND,
                    MANIFEST_INFO_COMMAND,
                ]
                .map(str::to_owned)
                .to_vec(),
                ..ExecuteCommandOptions::default()
            }),
            workspace: Some(lsp_types::WorkspaceServerCapabilities {
                workspace_folders: Some(lsp_types::WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(lsp_types::OneOf::Left(true)),
                }),
                ..lsp_types::WorkspaceServerCapabilities::default()
            }),
            ..ServerCapabilities::default()
        }
    }

    /// The checking options of one settings group: its folders, its
    /// exclusions and its open documents.
    pub(crate) fn workspace_checking_options(&self, group: usize) -> WorkspaceCheckingOptions {
        let roots = self
            .group_roots(group)
            .into_iter()
            .filter_map(|root| root.path.as_deref().map(std::path::PathBuf::from))
            .collect();
        let mut discovery = WorkspaceDiscoveryOptions::new(roots);
        discovery
            .exclusions
            .clone_from(&self.groups[group].settings.exclusions);
        let mut uris = self.document_uris();
        uris.sort();
        discovery.overlays = uris
            .into_iter()
            .filter(|uri| self.group_of(uri) == group)
            .filter_map(|uri| self.document_work(&uri).map(|work| work.input))
            .collect();
        WorkspaceCheckingOptions::new(discovery)
    }

    fn workspace_root(&self, document_uri: &str) -> Option<String> {
        let document_uri = document_uri.parse::<Uri>().ok()?;
        self.workspace_folders
            .iter()
            .filter(|root| root.contains(&document_uri))
            .max_by_key(|root| root.uri.path().as_str().len())
            .or_else(|| {
                self.root_uri
                    .as_ref()
                    .filter(|root| root.contains(&document_uri))
            })
            .and_then(|root| root.input_root.clone())
    }
}

mod configuration;
pub(crate) use configuration::{SECTIONS, workspace_exclusions};
mod groups;
pub(crate) use groups::FolderSettings;
use groups::SessionGroup;
mod documents;
pub(crate) use documents::resolved_document;
mod edits;
mod presentation;
pub(crate) use presentation::into_lsp_code_actions;
pub use presentation::into_lsp_range;

#[cfg(test)]
mod tests;
