use std::collections::HashMap;
use std::path::Path;

use lsp_types::{
    CodeLens, CodeLensOptions, Diagnostic, ExecuteCommandOptions, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
    WorkspaceFolder,
};
use serde::{Deserialize, Serialize};
use versionlens_engine::{WorkspaceCheck, WorkspaceCheckingOptions, WorkspaceDiscoveryOptions};
use versionlens_model::DocumentInput;

/// Updates one dependency: `{ uri, dependency, level?, version? }`.
pub(crate) const UPDATE_COMMAND: &str = "versionlens.server.update";
/// Updates every dependency of a document at a level: `{ uri, level? }`.
pub(crate) const UPDATE_ALL_COMMAND: &str = "versionlens.server.updateAll";
/// Updates every manifest in the workspace folders at a level: `{ level? }`.
pub(crate) const UPDATE_WORKSPACE_COMMAND: &str = "versionlens.server.updateWorkspace";
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
            .and_then(versionlens_engine::workspace_file_uri);
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
    let path = versionlens_engine::workspace_path(uri)?;
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
    /// Asked for a credential when a registry refuses a request.
    authorization: groups::AuthorizationSlot,
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
    pub(crate) fn with_config(self, config: versionlens_engine::SessionConfig) -> Self {
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
                    UPDATE_WORKSPACE_COMMAND,
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

    /// The checking options of one settings group: its open documents, and
    /// its folders and exclusions when the whole workspace is checked.
    pub(crate) fn workspace_checking_options(&self, group: usize) -> WorkspaceCheckingOptions {
        let scan = self.groups[group].settings.workspace.check == WorkspaceCheck::Workspace;
        self.workspace_discovery_options(group, scan)
    }

    /// The discovery of one settings group. Without `scan`, only its open
    /// documents are read.
    pub(crate) fn workspace_discovery_options(
        &self,
        group: usize,
        scan: bool,
    ) -> WorkspaceCheckingOptions {
        let mut discovery = self.workspace_scope(group);
        if !scan {
            discovery.roots.clear();
        }
        let mut uris = self.document_uris();
        uris.sort();
        discovery.overlays = uris
            .into_iter()
            .filter(|uri| self.group_of(uri) == group)
            .filter_map(|uri| self.document_work(&uri).map(|work| work.input))
            .collect();
        WorkspaceCheckingOptions::new(discovery)
    }

    /// The folders and exclusions of one settings group, without documents.
    pub(crate) fn workspace_scope(&self, group: usize) -> WorkspaceDiscoveryOptions {
        let settings = &self.groups[group].settings;
        let roots = self
            .group_roots(group)
            .into_iter()
            .filter_map(|root| root.path.as_deref().map(std::path::PathBuf::from))
            .collect();
        let mut discovery = WorkspaceDiscoveryOptions::new(roots);
        discovery.exclusions = [&settings.exclusions, &settings.workspace.exclude]
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        discovery
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
mod presentation;
pub(crate) use presentation::into_lsp_code_actions;
pub use presentation::into_lsp_range;

#[cfg(test)]
mod fixtures {
//! The group 2.2 regression fixtures, checked through the server. They share
//! manifests and registry answers with the engine fixtures
//! (`versionlens_test_support::regression`) and expect the same lenses and
//! diagnostics, so every client of the server (`JetBrains`, Zed, Neovim) gets
//! the engine's behavior.

use lsp_types::{CodeLens, Diagnostic, NumberOrString};
use versionlens_engine::{RegistryResponseInput, Settings, UPDATE_AVAILABLE};
use versionlens_test_support::regression::{self, Case};

use super::{VersionLensLspState, VersionLensTextDocument};

struct Served {
    lenses: Vec<CodeLens>,
    diagnostics: Vec<Diagnostic>,
}

impl Served {
    fn titles(&self) -> Vec<&str> {
        self.lenses
            .iter()
            .filter_map(|lens| lens.command.as_ref())
            .map(|command| command.title.as_str())
            .collect()
    }

    fn bumps(&self) -> Vec<&str> {
        self.titles()
            .into_iter()
            .filter(|title| title.starts_with('↑'))
            .collect()
    }

    /// The versions the bump lenses offer, from their command arguments. These
    /// are bare versions; the range operator is added when an update applies.
    fn bump_versions(&self) -> Vec<String> {
        self.lenses
            .iter()
            .filter_map(|lens| lens.command.as_ref())
            .filter(|command| command.title.starts_with('↑'))
            .filter_map(|command| command.arguments.as_ref()?.first()?.get("version"))
            .filter_map(|version| version.as_str().map(str::to_owned))
            .collect()
    }

    /// A bump lens sits on the given line, where the edit lands.
    fn assert_bump_on_line(&self, line: u32) {
        assert!(
            self.lenses.iter().any(|lens| {
                lens.range.start.line == line
                    && lens
                        .command
                        .as_ref()
                        .is_some_and(|command| command.title.starts_with('↑'))
            }),
            "no bump on line {line}: {:?}",
            self.lenses
        );
    }

    fn assert_status(&self, status: &str) {
        assert!(
            self.titles()
                .iter()
                .any(|title| title.ends_with(status) && !title.starts_with('↑')),
            "expected status `{status}`, lenses: {:?}",
            self.titles()
        );
    }

    fn assert_no_bump(&self) {
        assert_eq!(self.bumps(), Vec::<&str>::new());
    }

    /// No error lens (an error shows as a lens, never as a diagnostic) and no
    /// diagnostic other than an available update.
    fn assert_silent(&self) {
        assert!(
            self.titles()
                .iter()
                .all(|title| !title.starts_with('\u{1F534}')),
            "error lens: {:?}",
            self.titles()
        );
        let noise = self
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code != Some(NumberOrString::String(UPDATE_AVAILABLE.to_owned()))
            })
            .collect::<Vec<_>>();
        assert_eq!(noise, Vec::<&Diagnostic>::new());
    }
}

/// Registries that refuse connections, so any lookup surfaces as an error
/// instead of reaching the network.
fn offline() -> serde_json::Value {
    let unreachable = serde_json::json!({"apiUrl": "http://127.0.0.1:9/"});
    serde_json::json!({
        "github": unreachable,
        "npm": unreachable,
        "dotnet": {"nuget": {"sources": ["http://127.0.0.1:9/"]}},
    })
}

/// Opens the fixture in the server and resolves it with its registry answers.
async fn serve(case: &Case, offline_registries: bool) -> Served {
    let mut settings = if offline_registries {
        offline()
    } else {
        serde_json::json!({})
    };
    settings["suggestions"] = serde_json::json!({"showVulnerabilities": false});
    let config = Settings::from_value(settings)
        .expect("fixture settings are valid")
        .session_config();
    let mut state = VersionLensLspState::standard().with_config(config);
    state.open_document(VersionLensTextDocument {
        uri: case.uri.to_owned(),
        language_id: case.language.to_owned(),
        text: case.text.clone(),
        workspace_root: None,
    });
    let responses = case
        .registry
        .iter()
        .map(|stub| {
            let ecosystem = serde_json::from_value(serde_json::json!(stub.ecosystem))
                .unwrap_or_else(|error| panic!("{}: {error}", stub.ecosystem));
            RegistryResponseInput::new(stub.package, ecosystem, stub.body.clone())
        })
        .collect::<Vec<_>>();
    let work = state.document_work(case.uri).expect("document is open");
    state
        .session_for(case.uri)
        .resolve_document_with_responses(work.input, &responses)
        .await;
    let resolved = state.analyzed_document(case.uri).expect("document is open");
    Served {
        lenses: resolved.code_lenses,
        diagnostics: resolved.diagnostics,
    }
}

#[tokio::test]
async fn cargo_exact_pin_is_current() {
    let served = serve(&regression::cargo_exact_pin(), false).await;
    served.assert_status("latest 0.7.0");
    served.assert_no_bump();
}

#[tokio::test]
async fn python_exact_pin_is_current() {
    let served = serve(&regression::python_exact_pin(), false).await;
    served.assert_status("latest 2.34.2");
    served.assert_no_bump();
}

#[tokio::test]
async fn caret_range_offers_a_floor_bump_without_a_diagnostic() {
    let served = serve(&regression::caret_range_behind_latest(), false).await;
    served.assert_status("satisfies latest 2.5.14");
    assert_eq!(served.bump_versions(), ["2.5.14"]);
    assert_eq!(served.diagnostics, vec![]);
}

#[tokio::test]
async fn starlight_intermediate_releases_are_not_listed() {
    let served = serve(&regression::intermediate_releases(), false).await;
    served.assert_status("satisfies 0.41.11");
    assert_eq!(served.bump_versions(), ["0.41.11", "0.42.6"]);
    for skipped in regression::STARLIGHT_SKIPPED {
        assert!(
            served.titles().iter().all(|title| !title.contains(skipped)),
            "{skipped} listed: {:?}",
            served.titles()
        );
    }
}

#[tokio::test]
async fn github_script_sha_with_floating_and_exact_tags_is_current() {
    let served = serve(&regression::sha_pinned_action(), false).await;
    served.assert_status("latest v9.0.0");
    served.assert_no_bump();
}

#[tokio::test]
async fn install_action_branch_ref_is_fixed_without_a_lookup() {
    let served = serve(&regression::branch_action_ref(), true).await;
    served.assert_silent();
    served.assert_status("fixed just");
    served.assert_no_bump();
}

#[tokio::test]
async fn engines_vscode_is_skipped_without_an_error() {
    let served = serve(&regression::vscode_engine_field(), true).await;
    served.assert_silent();
    // The field is recognised and shown as unsupported, not looked up.
    assert_eq!(served.titles(), ["⚪ not supported"]);
}

#[tokio::test]
async fn setup_dotnet_unsupported_syntax_is_not_an_error() {
    let served = serve(&regression::unsupported_setup_dotnet(), true).await;
    served.assert_silent();
    served.assert_status("not supported");
    served.assert_no_bump();
}

#[tokio::test]
async fn env_reference_resolves_the_runtime() {
    let served = serve(&regression::workflow_env_reference(), false).await;
    served.assert_silent();
    // The runtime resolved from `${{ env.NODE_VERSION }}` ("20") against the
    // node release list.
    served.assert_status("fixed 20");
    assert_eq!(served.bump_versions(), ["22.0.0"]);
}

#[tokio::test]
async fn steps_output_expression_is_skipped_silently() {
    let served = serve(&regression::step_output_expression(), true).await;
    served.assert_silent();
    // Only the action itself is checked; the expression is not looked up.
    assert_eq!(served.titles().len(), 1, "{:?}", served.titles());
    served.assert_status("latest v4");
    served.assert_no_bump();
}

#[tokio::test]
async fn gradle_version_alias_resolves_through_its_library() {
    let served = serve(&regression::gradle_version_alias(), false).await;
    served.assert_silent();
    assert_eq!(served.bump_versions(), ["2.1.0"]);
    served.assert_bump_on_line(1);
}

#[tokio::test]
async fn gradle_plugin_alias_uses_the_plugin_marker() {
    let served = serve(&regression::gradle_plugin_alias(), false).await;
    served.assert_silent();
    assert_eq!(served.bump_versions(), ["2.0.0"]);
    served.assert_bump_on_line(1);
}

#[tokio::test]
async fn cargo_bare_version_is_a_caret_requirement() {
    let served = serve(&regression::cargo_bare_version(), false).await;
    served.assert_status("satisfies latest 1.0.229");
    assert_eq!(served.bump_versions(), ["1.0.229"]);
}

#[tokio::test]
async fn package_version_fields_are_not_dependencies() {
    for case in regression::package_version_fields() {
        let served = serve(&case, true).await;
        assert_eq!(served.titles(), Vec::<&str>::new(), "{}", case.uri);
        assert_eq!(served.diagnostics, vec![], "{}", case.uri);
    }
}
}
#[cfg(test)]
mod tests {
use lsp_types::{TextDocumentSyncCapability, TextDocumentSyncKind, Uri, WorkspaceFolder};

use super::{VersionLensLspState, VersionLensTextDocument};

type FileTestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn capabilities_match_implemented_document_and_command_protocol() -> Result<(), &'static str> {
    let capabilities = VersionLensLspState::server_capabilities();
    let Some(TextDocumentSyncCapability::Options(sync)) = capabilities.text_document_sync else {
        return Err("expected explicit text document sync options");
    };
    assert_eq!(sync.open_close, Some(true));
    assert_eq!(sync.change, Some(TextDocumentSyncKind::FULL));
    let Some(code_lens) = capabilities.code_lens_provider else {
        return Err("expected CodeLens capabilities");
    };
    assert_eq!(code_lens.resolve_provider, Some(false));
    let Some(execute_command) = capabilities.execute_command_provider else {
        return Err("expected execute command capabilities");
    };
    assert_eq!(
        execute_command.commands,
        [
            "versionlens.displayCodeLens",
            "versionlens.server.update",
            "versionlens.server.updateAll",
            "versionlens.server.updateWorkspace",
            "versionlens.server.sort",
            "versionlens.server.chooseBuild",
            "versionlens.server.clearCache",
            "versionlens.server.manifestInfo",
        ]
    );
    Ok(())
}

#[test]
fn selects_the_deepest_workspace_folder_and_decodes_its_path() -> Result<(), String> {
    let root_uri = uri("file:///workspace")?;
    let workspace_folders = vec![
        folder("file:///workspace/project", "project")?,
        folder("file:///workspace/project/nested%20folder", "nested")?,
    ];
    let mut state = VersionLensLspState::with_workspace(Some(root_uri), workspace_folders);
    let document_uri = "file:///workspace/project/nested%20folder/package.json";

    state.open_document(document(document_uri, None));

    assert_eq!(
        state
            .documents
            .get(document_uri)
            .and_then(|document| document.workspace_root.as_deref()),
        Some("file:///workspace/project/nested%20folder")
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn workspace_roots_match_canonical_document_uris_and_new_files() -> FileTestResult {
    use versionlens_model::{
        DocumentEditPlan, DocumentSnapshot, WorkspaceEditPlan, document_text_hash,
    };

    let directory =
        versionlens_test_support::temporary_directory("versionlens-lsp-roots")?.canonicalize()?;
    let real = directory.join("real");
    let alias = directory.join("alias");
    std::fs::create_dir(&real)?;
    std::os::unix::fs::symlink(&real, &alias)?;
    let root_uri = format!("file://{}", alias.display());
    let mut state = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), vec![]);
    let document_uri = format!("file://{}/package.json", alias.display());
    let canonical_uri = format!("file://{}/package.json", real.display());
    state.open_document(document(&document_uri, None));
    let work = state.document_work(&document_uri).unwrap();
    assert_eq!(
        work.input.workspace_root.as_deref(),
        versionlens_engine::workspace_file_uri(&real).as_deref()
    );
    assert_eq!(work.input.uri, canonical_uri);
    assert_eq!(
        state.document_work(&canonical_uri).unwrap().uri,
        document_uri
    );
    let edit = state.workspace_edit(&WorkspaceEditPlan {
        documents: vec![DocumentEditPlan {
            document: DocumentSnapshot {
                uri: canonical_uri,
                version: None,
                text_hash: document_text_hash("{}"),
            },
            edits: vec![],
        }],
    })?;
    assert_eq!(
        edit["documentChanges"][0]["textDocument"]["uri"],
        document_uri
    );
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn root_uri_is_used_only_for_documents_inside_it() -> Result<(), String> {
    let mut state =
        VersionLensLspState::with_workspace(Some(uri("file:///workspace")?), Vec::new());
    let inside = "file:///workspace/package.json";
    let outside = "file:///workspace-other/package.json";

    state.open_document(document(inside, None));
    state.open_document(document(outside, None));

    assert_eq!(
        state
            .documents
            .get(inside)
            .and_then(|document| document.workspace_root.as_deref()),
        Some("file:///workspace")
    );
    assert_eq!(
        state
            .documents
            .get(outside)
            .and_then(|document| document.workspace_root.as_deref()),
        None
    );
    Ok(())
}

#[test]
fn existing_file_workspace_root_uses_its_parent_directory() -> FileTestResult {
    let directory = versionlens_test_support::temporary_directory("versionlens-lsp-file-root")?
        .canonicalize()?;
    let manifest = directory.join("package.json");
    std::fs::write(&manifest, "{}")?;
    let manifest_uri =
        versionlens_engine::workspace_file_uri(&manifest).ok_or("missing file URI")?;
    let directory_uri =
        versionlens_engine::workspace_file_uri(&directory).ok_or("missing directory URI")?;
    let mut state = VersionLensLspState::with_workspace(Some(uri(&manifest_uri)?), Vec::new())
        .with_settings(workspace_check());

    state.open_document(document(&manifest_uri, None));

    assert_eq!(
        state
            .document_work(&manifest_uri)
            .unwrap()
            .input
            .workspace_root,
        Some(directory_uri)
    );
    assert_eq!(
        state.workspace_checking_options(0).discovery.roots,
        [directory]
    );
    Ok(())
}

#[test]
fn workspace_folder_changes_rebind_open_documents_and_invalidate_work() -> Result<(), String> {
    let parent = folder("file:///workspace", "parent")?;
    let nested = folder("file:///workspace/nested%20folder", "nested")?;
    let mut state =
        VersionLensLspState::with_workspace(Some(parent.uri.clone()), vec![parent.clone()]);
    let document_uri = "file:///workspace/nested%20folder/package.json";
    state.open_document(document(document_uri, None));
    state.set_document_version(document_uri, 7);
    for (added, removed, expected) in [
        (
            vec![nested.clone()],
            vec![],
            Some("file:///workspace/nested%20folder"),
        ),
        (vec![], vec![nested], Some("file:///workspace")),
        (vec![], vec![parent], None),
    ] {
        let before = state
            .document_work(document_uri)
            .ok_or("missing open document")?;
        state
            .change_workspace_folders(lsp_types::WorkspaceFoldersChangeEvent { added, removed })
            .map_err(|error| error.to_string())?;
        let after = state
            .document_work(document_uri)
            .ok_or("missing updated document")?;
        assert!(!state.work_is_current(&before));
        assert_eq!(after.input.workspace_root.as_deref(), expected);
        assert_eq!(after.version, Some(7));
    }
    Ok(())
}

#[test]
fn code_lenses_bind_updates_to_the_document_and_close_with_it() {
    let mut state = VersionLensLspState::standard();
    let uri = "file:///workspace/jsr.json";
    state.open_document(VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"name":"@scope/pkg","version":"1.0.0"}"#.to_owned(),
        workspace_root: Some("/workspace".to_owned()),
    });

    let lenses = state.code_lenses(uri);
    crate::test_support::assert_update_code_lenses(&lenses);

    state.close_document(uri);
    assert!(state.code_lenses(uri).is_empty());
    assert!(!state.documents.contains_key(uri));
}

#[tokio::test]
async fn dependency_lenses_share_the_meaningful_source_range() {
    let mut state = VersionLensLspState::standard();
    let uri = "file:///workspace/package.json";
    let text = r#"{"dependencies":{"example":"1.0.0"}}"#;
    state.open_document(VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: Some("/workspace".to_owned()),
    });
    let work = state.document_work(uri).unwrap();
    state
        .session_for(uri)
        .resolve_document_with_responses(
            work.input,
            &[versionlens_engine::RegistryResponseInput::new(
                "example",
                versionlens_model::Ecosystem::Npm,
                r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#,
            )],
        )
        .await;

    let lenses = state.code_lenses(uri);

    assert!(lenses.len() >= 2);
    assert!(lenses.iter().all(|lens| {
        lens.range
            == lsp_types::Range::new(
                lsp_types::Position::new(0, 18),
                lsp_types::Position::new(0, 33),
            )
    }));
}

#[test]
fn changing_an_unknown_document_does_not_open_it() {
    let mut state = VersionLensLspState::standard();
    let uri = "file:///missing/package.json";
    state.change_document(uri, "{}".to_owned());
    assert!(state.document_work(uri).is_none());
}

fn uri(value: &str) -> Result<Uri, String> {
    value.parse::<Uri>().map_err(|error| error.to_string())
}

fn folder(value: &str, name: &str) -> Result<WorkspaceFolder, String> {
    Ok(WorkspaceFolder {
        uri: uri(value)?,
        name: name.to_owned(),
    })
}

fn document(uri: &str, workspace_root: Option<String>) -> VersionLensTextDocument {
    VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "json".to_owned(),
        text: "{}".to_owned(),
        workspace_root,
    }
}

#[test]
fn configuration_changes_invalidate_document_work_and_preserve_versions() -> Result<(), String> {
    let mut state = VersionLensLspState::standard();
    let uri = "file:///workspace/package.json";
    state.open_document(document(uri, None));
    state.set_document_version(uri, 9);
    let work = state.document_work(uri).ok_or("missing document")?;
    let config = versionlens_engine::Settings::from_value(
        serde_json::json!({"suggestions":{"showVulnerabilities":false}}),
    )?
    .session_config();
    assert!(!config.show_vulnerabilities);
    let changed = state
        .replace_settings(vec![super::FolderSettings {
            config,
            ..super::FolderSettings::default()
        }])
        .map_err(|error| error.to_string())?;
    assert!(changed);
    assert!(!state.work_is_current(&work));
    assert_eq!(
        state.document_work(uri).ok_or("missing document")?.version,
        Some(9)
    );
    assert!(!state.accepts_document_version(uri, 9));
    assert!(state.accepts_document_version(uri, 10));
    Ok(())
}

#[test]
fn workspace_edits_validate_every_open_document_before_application() {
    use versionlens_model::{
        DocumentEditPlan, DocumentSnapshot, WorkspaceEditPlan, document_text_hash,
    };
    let mut state = VersionLensLspState::standard();
    let uris = [
        "file:///workspace/a/package.json",
        "file:///workspace/b/package.json",
    ];
    for uri in uris {
        state.open_document(document(uri, None));
        state.set_document_version(uri, 1);
    }
    let plan = WorkspaceEditPlan {
        documents: uris
            .into_iter()
            .map(|uri| DocumentEditPlan {
                document: DocumentSnapshot {
                    uri: uri.to_owned(),
                    version: Some(1),
                    text_hash: document_text_hash("{}"),
                },
                edits: vec![],
            })
            .collect(),
    };
    assert!(state.workspace_edit(&plan).is_ok());
    state.change_document(uris[1], "{\"version\":\"2.0.0\"}".to_owned());
    state.set_document_version(uris[1], 2);
    assert!(state.workspace_edit(&plan).is_err());
}

#[test]
fn workspace_snapshot_changes_invalidate_other_open_document_work() {
    let root = std::env::temp_dir().canonicalize().unwrap();
    let mut state = VersionLensLspState::standard();
    let paths = [
        "versionlens-lsp-a/package.json",
        "versionlens-lsp-b/package.json",
    ]
    .map(|name| root.join(name).to_string_lossy().into_owned());
    for path in &paths {
        state.open_document(document(path, Some(root.to_string_lossy().into_owned())));
    }
    let work = state.document_work(&paths[0]).unwrap();
    state.change_document(&paths[1], "{\"version\":\"2.0.0\"}".to_owned());
    assert!(!state.work_is_current(&work));
    let work = state.document_work(&paths[0]).unwrap();
    state.close_document(&paths[1]);
    assert!(!state.work_is_current(&work));
    let work = state.document_work(&paths[0]).unwrap();
    state.invalidate_workspace();
    assert!(!state.work_is_current(&work));
}

#[test]
fn editor_exclusions_expand_braces_and_filter_watched_files() -> Result<(), String> {
    let root = std::env::temp_dir()
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let root_uri = versionlens_engine::workspace_file_uri(&root).ok_or("missing root URI")?;
    let state = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), Vec::new())
        .with_workspace_exclusions(super::workspace_exclusions(&serde_json::json!({
            "**/{generated,cache}/**": true
        })));
    let included = root.join("src/package.json");
    let cargo = root.join("src/Cargo.toml");
    let configured = root.join("src/deps.custom");
    let unrelated = root.join("src/readme.md");
    let generated = root.join("generated/package.json");
    let cache = root.join("cache/package.json");
    let file_uri = |path: &std::path::Path| {
        versionlens_engine::workspace_file_uri(path).ok_or_else(|| "missing file URI".to_owned())
    };

    assert!(state.watched_file_is_relevant(&file_uri(&included)?));
    assert!(!state.watched_file_is_relevant(&file_uri(&unrelated)?));
    assert!(!state.watched_file_is_relevant(&file_uri(&generated)?));
    assert!(!state.watched_file_is_relevant(&file_uri(&cache)?));
    let options = state.workspace_checking_options(0);
    assert!(options.discovery.provider_exclusions.is_empty());
    assert!(
        options
            .discovery
            .exclusions
            .contains(&"**/obj/**".to_owned())
    );

    let cargo_only = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), Vec::new())
        .with_config(
            versionlens_engine::Settings::from_value(serde_json::json!({
                "enabledProviders": ["cargo"]
            }))?
            .session_config(),
        );
    assert!(cargo_only.watched_file_is_relevant(&file_uri(&cargo)?));
    assert!(!cargo_only.watched_file_is_relevant(&file_uri(&included)?));

    let configured_npm = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), Vec::new())
        .with_config(
            versionlens_engine::Settings::from_value(serde_json::json!({
                "enabledProviders": ["npm"],
                "npm": {"files": "**/deps.custom"}
            }))?
            .session_config(),
        );
    assert!(configured_npm.watched_file_is_relevant(&file_uri(&configured)?));
    Ok(())
}

#[test]
fn only_ignore_files_that_discovery_reads_are_watched() -> Result<(), String> {
    let root = versionlens_test_support::temporary_directory("versionlens-watched-ignore")
        .map_err(|error| error.to_string())?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    std::fs::write(root.join(".gitignore"), "worktrees/\n").map_err(|error| error.to_string())?;
    let root_uri = versionlens_engine::workspace_file_uri(&root).ok_or("missing root URI")?;
    let open_files = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), Vec::new());
    let gitignore =
        versionlens_engine::workspace_file_uri(&root.join(".gitignore")).ok_or("missing URI")?;
    assert!(open_files.watched_ignore_file(&gitignore));
    let workspace = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), Vec::new())
        .with_settings(workspace_check());
    for state in [&open_files, &workspace] {
        let watched = |relative: &str| {
            versionlens_engine::workspace_file_uri(&root.join(relative))
                .is_some_and(|uri| state.watched_ignore_file(&uri))
        };

        assert!(watched(".gitignore"));
        assert!(watched("app/.ignore"));
        assert!(watched(".git/info/exclude"));
        assert!(!watched("worktrees/copy/.gitignore"));
        assert!(!watched("node_modules/pkg/.gitignore"));
        assert!(!watched("app/package.json"));
    }
    std::fs::remove_dir_all(root).map_err(|error| error.to_string())
}

#[test]
fn folders_that_differ_only_in_exclusions_get_their_own_sessions() -> Result<(), String> {
    let mut state = VersionLensLspState::with_workspace(
        None,
        vec![
            folder("file:///first", "first")?,
            folder("file:///second", "second")?,
        ],
    );
    let mut second = super::FolderSettings::default();
    second.exclusions.push("**/generated/**".to_owned());
    state
        .replace_settings(vec![super::FolderSettings::default(), second])
        .map_err(|error| error.to_string())?;
    let [first, second] = state.groups() else {
        return Err(format!("expected two groups, got {}", state.groups().len()));
    };
    let documents = vec![versionlens_model::DocumentInput::new(
        "file:///first/package.json".to_owned(),
        "json".to_owned(),
        "{}".to_owned(),
        None,
    )];
    assert!(first.session.set_workspace_documents(documents.clone()));
    // A shared session would already hold these documents.
    assert!(second.session.set_workspace_documents(documents));
    Ok(())
}

#[test]
fn code_actions_use_plain_changes_for_clients_without_document_changes() {
    let range = versionlens_model::Range {
        start: versionlens_model::Position {
            line: 0,
            character: 1,
        },
        end: versionlens_model::Position {
            line: 0,
            character: 2,
        },
    };
    let action = versionlens_engine::BumpActionView {
        title: "Bump to ^2.0.0".to_owned(),
        edit: versionlens_model::TextEdit {
            range,
            new_text: "^2.0.0".to_owned(),
        },
        preferred: true,
        diagnostic: None,
    };
    let uri = "file:///workspace/package.json"
        .parse::<lsp_types::Uri>()
        .unwrap();

    let actions = super::into_lsp_code_actions(vec![action], &uri, Some(3), false);

    let value = serde_json::to_value(&actions).unwrap();
    assert_eq!(
        value[0]["edit"],
        serde_json::json!({"changes": {"file:///workspace/package.json": [{
            "range": {"start": {"line": 0, "character": 1}, "end": {"line": 0, "character": 2}},
            "newText": "^2.0.0"
        }]}})
    );
    assert_eq!(value[0].get("diagnostics"), None);
}

fn workspace_check() -> super::FolderSettings {
    super::FolderSettings {
        workspace: versionlens_engine::WorkspaceSettings {
            check: versionlens_engine::WorkspaceCheck::Workspace,
            exclude: vec!["fixtures/**".to_owned()],
        },
        ..super::FolderSettings::default()
    }
}

#[test]
fn only_the_workspace_check_scans_folders() -> Result<(), String> {
    let mut state =
        VersionLensLspState::with_workspace(Some(uri("file:///workspace")?), Vec::new());
    assert!(
        state
            .workspace_checking_options(0)
            .discovery
            .roots
            .is_empty()
    );
    assert_eq!(
        state
            .workspace_discovery_options(0, true)
            .discovery
            .roots
            .len(),
        1
    );
    state = state.with_settings(workspace_check());
    let discovery = state.workspace_checking_options(0).discovery;
    assert_eq!(discovery.roots, [std::path::PathBuf::from("/workspace")]);
    assert!(discovery.exclusions.contains(&"fixtures/**".to_owned()));
    Ok(())
}

#[test]
fn open_documents_that_a_scan_skips_are_not_checked() -> Result<(), String> {
    let root = versionlens_test_support::temporary_directory("versionlens-open-ignored")
        .map_err(|error| error.to_string())?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    std::fs::write(root.join(".gitignore"), ".claude/worktrees/\n")
        .map_err(|error| error.to_string())?;
    let root_uri = versionlens_engine::workspace_file_uri(&root).ok_or("missing root URI")?;
    let root_uri = uri(&root_uri)?;
    let with_files_exclude = |mut settings: super::FolderSettings| {
        settings.exclusions.push("**/generated/**".to_owned());
        settings.exclusions.push("**/scratch".to_owned());
        VersionLensLspState::with_workspace(Some(root_uri.clone()), Vec::new())
            .with_settings(settings)
    };
    for (mut state, mode) in [
        (
            with_files_exclude(super::FolderSettings::default()),
            "open files",
        ),
        (with_files_exclude(workspace_check()), "workspace"),
    ] {
        for (path, checked) in [
            (".claude/worktrees/agent/launcher/Cargo.toml", false),
            ("generated/Cargo.toml", false),
            ("scratch/tool/Cargo.toml", false),
            ("build/tool/Cargo.toml", true),
            ("fixtures/Cargo.toml", mode == "open files"),
            ("app/Cargo.toml", true),
        ] {
            let document = versionlens_engine::workspace_file_uri(&root.join(path))
                .ok_or("missing document URI")?;
            state.open_document(super::VersionLensTextDocument {
                uri: document.clone(),
                language_id: "toml".to_owned(),
                text: "[dependencies]\n".to_owned(),
                workspace_root: None,
            });
            assert_eq!(
                state.document_work(&document).is_some(),
                checked,
                "{mode}: {path}"
            );
        }
    }
    std::fs::remove_dir_all(root).map_err(|error| error.to_string())
}
}
