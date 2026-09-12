use std::collections::HashMap;
use std::path::Path;

use lsp_types::{
    CodeLens, CodeLensOptions, Diagnostic, ExecuteCommandOptions, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
    WorkspaceFolder,
};
use serde::{Deserialize, Serialize};
use versionlens_core::{
    SessionConfigInput, VersionLensSession, WorkspaceCheckingOptions, WorkspaceDiscoveryOptions,
    WorkspaceProviderExclusion, version_lens_session,
};
use versionlens_model::{DocumentInput, Ecosystem};

pub(crate) const UPDATE_DEPENDENCY_COMMAND: &str = "versionlens.suggestion.onUpdateDependency";

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
    pub(crate) session: VersionLensSession,
    configuration: versionlens_core::SessionConfig,
    documents: HashMap<String, VersionLensTextDocument>,
    revisions: HashMap<String, DocumentRevision>,
    generation: u64,
    root_uri: Option<WorkspaceRoot>,
    workspace_folders: Vec<WorkspaceRoot>,
    exclusions: Vec<String>,
    pub(crate) client: ClientSupport,
    modes: StateModes,
}

#[derive(Debug, Default)]
struct StateModes {
    persistent: bool,
    controller_managed: bool,
}

#[derive(Debug, Default)]
pub(crate) struct ClientSupport {
    pub(crate) workspace_edits: bool,
    pub(crate) code_lens_refresh: bool,
    pub(crate) watched_files_dynamic: bool,
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

pub(crate) struct ResolvedDocument {
    pub(crate) code_lenses: Vec<CodeLens>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

pub(crate) struct CheckedDocument {
    pub(crate) uri: String,
    pub(crate) version: Option<i32>,
    pub(crate) generation: u64,
    pub(crate) input: DocumentInput,
    pub(crate) open: bool,
}

impl VersionLensLspState {
    pub(crate) fn with_config(mut self, config: versionlens_core::SessionConfig) -> Self {
        self.session = version_lens_session(config.clone());
        self.configuration = config;
        self
    }

    pub(crate) fn with_client(mut self, capabilities: lsp_types::ClientCapabilities) -> Self {
        if let Some(workspace) = capabilities.workspace {
            self.client.workspace_edits = workspace.apply_edit == Some(true)
                && workspace
                    .workspace_edit
                    .is_some_and(|edit| edit.document_changes == Some(true));
            self.client.code_lens_refresh = workspace
                .code_lens
                .is_some_and(|lens| lens.refresh_support == Some(true));
            self.client.watched_files_dynamic = workspace
                .did_change_watched_files
                .is_some_and(|files| files.dynamic_registration == Some(true));
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
        let configuration: versionlens_core::SessionConfig = SessionConfigInput::default().into();
        Self {
            session: version_lens_session(configuration.clone()),
            configuration,
            documents: HashMap::new(),
            revisions: HashMap::new(),
            generation: 0,
            client: ClientSupport::default(),
            modes: StateModes::default(),
            root_uri: root_uri.map(WorkspaceRoot::new),
            workspace_folders: workspace_folders
                .into_iter()
                .map(|folder| WorkspaceRoot::new(folder.uri))
                .collect(),
            exclusions: configuration::default_workspace_exclusions(),
        }
    }

    pub(crate) fn with_application_cache(mut self) -> std::io::Result<Self> {
        self.session = self.session.with_application_cache()?;
        self.modes.persistent = true;
        Ok(self)
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
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec![
                    DISPLAY_CODE_LENS_COMMAND.to_owned(),
                    UPDATE_DEPENDENCY_COMMAND.to_owned(),
                ],
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

    pub(crate) fn workspace_checking_options(&self) -> WorkspaceCheckingOptions {
        let roots = if self.workspace_folders.is_empty() {
            self.root_uri
                .iter()
                .filter_map(|root| root.path.as_deref().map(std::path::PathBuf::from))
                .collect()
        } else {
            self.workspace_folders
                .iter()
                .filter_map(|root| root.path.as_deref().map(std::path::PathBuf::from))
                .collect()
        };
        let mut discovery = WorkspaceDiscoveryOptions::new(roots);
        discovery.exclusions = self.exclusions.clone();
        discovery.provider_exclusions = vec![WorkspaceProviderExclusion {
            ecosystem: Ecosystem::Dotnet,
            patterns: vec!["**/obj/**".to_owned()],
        }];
        let mut uris = self.document_uris();
        uris.sort();
        discovery.overlays = uris
            .into_iter()
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
pub(crate) use configuration::{session_configuration, workspace_exclusions};
mod documents;
mod edits;
mod presentation;
pub use presentation::into_lsp_range;

#[cfg(test)]
mod tests;
