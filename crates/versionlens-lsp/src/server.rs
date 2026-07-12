use anyhow::{Context, Result};
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{DidChangeTextDocument, DidOpenTextDocument, Notification as _};
use lsp_types::request::{CodeLensRequest, Request as LspRequest};
use lsp_types::{
    CodeLens, CodeLensOptions, Command, Diagnostic, DiagnosticSeverity, NumberOrString, Position,
    PublishDiagnosticsParams, Range, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Uri,
};
use lsp_types::{CodeLensParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use versionlens_core::{SessionConfigInput, VersionLensSession, version_lens_session};
use versionlens_parsers::DocumentInput;
use versionlens_vscode_model::{CodeLensPayload, DiagnosticPayload};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionLensTextDocument {
    pub uri: String,
    pub language_id: String,
    pub text: String,
    pub workspace_root: Option<String>,
}

#[derive(Debug)]
pub struct VersionLensLspState {
    session: VersionLensSession,
    documents: HashMap<String, VersionLensTextDocument>,
}

impl VersionLensLspState {
    pub fn standard() -> Self {
        Self {
            session: version_lens_session(SessionConfigInput::default().into()),
            documents: HashMap::new(),
        }
    }

    pub fn server_capabilities() -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false),
            }),
            ..ServerCapabilities::default()
        }
    }

    pub fn open_document(&mut self, document: VersionLensTextDocument) -> Vec<Diagnostic> {
        let diagnostics = self.analyze_document(&document).diagnostics;
        self.documents
            .insert(String::from(document.uri.as_str()), document);
        diagnostics.into_iter().map(into_lsp_diagnostic).collect()
    }

    pub fn change_document(&mut self, uri: &str, text: String) -> Vec<Diagnostic> {
        let Some(existing) = self.documents.get(uri) else {
            return Vec::new();
        };
        let document = VersionLensTextDocument {
            uri: String::from(existing.uri.as_str()),
            language_id: String::from(existing.language_id.as_str()),
            text,
            workspace_root: existing.workspace_root.as_deref().map(str::to_string),
        };
        self.open_document(document)
    }

    pub fn code_lenses(&self, uri: &str) -> Vec<CodeLens> {
        let Some(document) = self.documents.get(uri) else {
            return Vec::new();
        };
        self.session.resolve_document(document_input(document));
        self.session
            .analyze_document(document_input(document))
            .code_lenses
            .into_iter()
            .map(into_lsp_code_lens)
            .collect()
    }

    pub fn publish_diagnostics(uri: Uri, diagnostics: Vec<Diagnostic>) -> PublishDiagnosticsParams {
        PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        }
    }

    fn analyze_document(
        &self,
        document: &VersionLensTextDocument,
    ) -> versionlens_core::AnalyzeDocumentOutput {
        self.session.analyze_document(document_input(document))
    }
}

fn document_input(document: &VersionLensTextDocument) -> DocumentInput {
    DocumentInput {
        uri: String::from(document.uri.as_str()),
        language_id: String::from(document.language_id.as_str()),
        text: String::from(document.text.as_str()),
        workspace_root: document.workspace_root.as_deref().map(str::to_string),
    }
}

pub fn into_lsp_range(range: versionlens_vscode_model::Range) -> Range {
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
            command: payload.command,
            arguments: Some(payload.arguments.into_iter().map(Value::String).collect()),
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

pub fn run_stdio_server() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    let initialize_value = serde_json::to_value(VersionLensLspState::server_capabilities())?;
    connection.initialize(initialize_value)?;

    let mut state = VersionLensLspState::standard();
    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    break;
                }
                handle_request(&connection, &state, request)?;
            }
            Message::Notification(notification) => {
                handle_notification(&connection, &mut state, notification)?;
            }
            Message::Response(_) => {}
        }
    }

    io_threads.join()?;
    Ok(())
}

fn handle_request(
    connection: &Connection,
    state: &VersionLensLspState,
    request: Request,
) -> Result<()> {
    let Request { id, method, params } = request;
    match method.as_str() {
        CodeLensRequest::METHOD => {
            let params: CodeLensParams = serde_json::from_value(params)?;
            let lenses = state.code_lenses(params.text_document.uri.as_str());
            respond(connection, id, serde_json::to_value(lenses)?)
        }
        _ => respond(connection, id, Value::Null),
    }
}

fn handle_notification(
    connection: &Connection,
    state: &mut VersionLensLspState,
    notification: Notification,
) -> Result<()> {
    match notification.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let params: DidOpenTextDocumentParams = serde_json::from_value(notification.params)?;
            let uri = params.text_document.uri;
            let diagnostics = state.open_document(VersionLensTextDocument {
                uri: uri.to_string(),
                language_id: params.text_document.language_id,
                text: params.text_document.text,
                workspace_root: None,
            });
            publish_diagnostics(connection, uri, diagnostics)
        }
        DidChangeTextDocument::METHOD => {
            let params: DidChangeTextDocumentParams = serde_json::from_value(notification.params)?;
            let uri = params.text_document.uri;
            let text = params
                .content_changes
                .into_iter()
                .next()
                .map(|change| change.text)
                .unwrap_or_default();
            let diagnostics = state.change_document(uri.as_str(), text);
            publish_diagnostics(connection, uri, diagnostics)
        }
        _ => Ok(()),
    }
}

fn respond(connection: &Connection, id: RequestId, result: serde_json::Value) -> Result<()> {
    connection
        .sender
        .send(Message::Response(Response {
            id,
            response_kind: lsp_server::ResponseKind::Ok { result },
        }))
        .context("failed to send LSP response")
}

fn publish_diagnostics(
    connection: &Connection,
    uri: Uri,
    diagnostics: Vec<lsp_types::Diagnostic>,
) -> Result<()> {
    let params = VersionLensLspState::publish_diagnostics(uri, diagnostics);
    connection
        .sender
        .send(Message::Notification(Notification {
            method: "textDocument/publishDiagnostics".to_owned(),
            params: serde_json::to_value(params)?,
        }))
        .context("failed to publish LSP diagnostics")
}

#[cfg(test)]
mod tests;
