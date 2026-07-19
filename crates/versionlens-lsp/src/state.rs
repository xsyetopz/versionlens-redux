use std::collections::HashMap;

use lsp_types::{
    CodeLens, CodeLensOptions, Command, Diagnostic, DiagnosticSeverity, ExecuteCommandOptions,
    NumberOrString, Position, PublishDiagnosticsParams, Range, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
    WorkspaceFolder,
};
use serde::{Deserialize, Serialize};
use versionlens_core::{SessionConfigInput, VersionLensSession, version_lens_session};
use versionlens_model::{DocumentInput, Range as ModelRange};
use versionlens_vscode_model::{CodeLensPayload, DiagnosticPayload};

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
}

impl WorkspaceRoot {
    fn new(uri: Uri) -> Self {
        let path = (uri.scheme().is_some_and(|scheme| scheme.as_str() == "file")).then(|| {
            uri.path()
                .as_estr()
                .decode()
                .into_string_lossy()
                .into_owned()
        });
        Self { uri, path }
    }

    fn contains(&self, document_uri: &Uri) -> bool {
        if self.uri.scheme().map(|scheme| scheme.as_str())
            != document_uri.scheme().map(|scheme| scheme.as_str())
            || self.uri.authority().map(|authority| authority.as_str())
                != document_uri.authority().map(|authority| authority.as_str())
        {
            return false;
        }
        let root = self.uri.path().as_str().trim_end_matches('/');
        let document = document_uri.path().as_str();
        document == root
            || document
                .strip_prefix(root)
                .is_some_and(|relative| relative.starts_with('/'))
    }
}

#[derive(Debug)]
pub struct VersionLensLspState {
    session: VersionLensSession,
    documents: HashMap<String, VersionLensTextDocument>,
    root_uri: Option<WorkspaceRoot>,
    workspace_folders: Vec<WorkspaceRoot>,
}

pub(crate) struct ResolvedDocument {
    pub(crate) code_lenses: Vec<CodeLens>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl VersionLensLspState {
    pub fn standard() -> Self {
        Self::with_workspace(None, Vec::new())
    }

    pub(crate) fn with_workspace(
        root_uri: Option<Uri>,
        workspace_folders: Vec<WorkspaceFolder>,
    ) -> Self {
        Self {
            session: version_lens_session(SessionConfigInput::default().into()),
            documents: HashMap::new(),
            root_uri: root_uri.map(WorkspaceRoot::new),
            workspace_folders: workspace_folders
                .into_iter()
                .map(|folder| WorkspaceRoot::new(folder.uri))
                .collect(),
        }
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
                commands: vec![DISPLAY_CODE_LENS_COMMAND.to_owned()],
                ..ExecuteCommandOptions::default()
            }),
            ..ServerCapabilities::default()
        }
    }

    pub fn open_document(&mut self, mut document: VersionLensTextDocument) -> Vec<Diagnostic> {
        if document.workspace_root.is_none() {
            document.workspace_root = self.workspace_root(&document.uri);
        }
        let diagnostics = self.analyze_document(&document).diagnostics;
        self.documents.insert(document.uri.clone(), document);
        diagnostics.into_iter().map(into_lsp_diagnostic).collect()
    }

    pub fn change_document(&mut self, uri: &str, text: String) -> Vec<Diagnostic> {
        let Some(existing) = self.documents.get(uri) else {
            return Vec::new();
        };
        let document = VersionLensTextDocument {
            uri: existing.uri.clone(),
            language_id: existing.language_id.clone(),
            text,
            workspace_root: existing.workspace_root.clone(),
        };
        self.open_document(document)
    }

    pub fn close_document(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    pub fn code_lenses(&self, uri: &str) -> Vec<CodeLens> {
        self.resolve_document(uri)
            .map_or_else(Vec::new, |resolved| resolved.code_lenses)
    }

    pub fn publish_diagnostics(uri: Uri, diagnostics: Vec<Diagnostic>) -> PublishDiagnosticsParams {
        PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        }
    }

    pub(crate) fn resolve_document(&self, uri: &str) -> Option<ResolvedDocument> {
        let document = self.documents.get(uri)?;
        self.session.resolve_document(document_input(document));
        let analysis = self.analyze_document(document);
        Some(ResolvedDocument {
            code_lenses: analysis
                .code_lenses
                .into_iter()
                .map(into_lsp_code_lens)
                .collect(),
            diagnostics: analysis
                .diagnostics
                .into_iter()
                .map(into_lsp_diagnostic)
                .collect(),
        })
    }

    fn analyze_document(
        &self,
        document: &VersionLensTextDocument,
    ) -> versionlens_core::AnalyzeDocumentOutput {
        self.session.analyze_document(document_input(document))
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
            .and_then(|root| root.path.clone())
    }
}

fn document_input(document: &VersionLensTextDocument) -> DocumentInput {
    DocumentInput {
        uri: document.uri.clone(),
        language_id: document.language_id.clone(),
        text: document.text.clone(),
        workspace_root: document.workspace_root.clone(),
    }
}

pub fn into_lsp_range(range: ModelRange) -> Range {
    Range {
        start: Position::new(range.start.line, range.start.character),
        end: Position::new(range.end.line, range.end.character),
    }
}

fn into_lsp_code_lens(payload: CodeLensPayload) -> CodeLens {
    CodeLens {
        range: into_lsp_range(payload.range),
        command: Some(Command {
            title: payload.title,
            command: DISPLAY_CODE_LENS_COMMAND.to_owned(),
            arguments: None,
        }),
        data: None,
    }
}

fn into_lsp_diagnostic(payload: DiagnosticPayload) -> Diagnostic {
    Diagnostic {
        range: into_lsp_range(payload.range),
        severity: diagnostic_severity(payload.severity),
        code: payload.code.map(NumberOrString::String),
        code_description: payload
            .code_description_url
            .and_then(|href| href.parse::<Uri>().ok())
            .map(|href| lsp_types::CodeDescription { href }),
        source: payload.source,
        message: payload.message,
        related_information: None,
        tags: None,
        data: None,
    }
}

fn diagnostic_severity(severity: u8) -> Option<DiagnosticSeverity> {
    match severity {
        0 => Some(DiagnosticSeverity::ERROR),
        1 => Some(DiagnosticSeverity::WARNING),
        2 => Some(DiagnosticSeverity::INFORMATION),
        3 => Some(DiagnosticSeverity::HINT),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
