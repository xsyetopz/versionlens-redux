use std::fs;

use lsp_types::{CodeLens, Diagnostic, PublishDiagnosticsParams, Uri};
use serde_json::{Value, json};
use versionlens_engine::AnalyzeDocumentOutput;
use versionlens_model::{DocumentInput, WorkspaceEditPlan, document_text_hash};

use super::presentation::{into_lsp_code_lenses, into_lsp_diagnostic};
use super::{
    CheckedDocument, DocumentRevision, DocumentWork, ResolvedDocument, VersionLensLspState,
    VersionLensTextDocument, normalized_file_path,
};

impl VersionLensLspState {
    /// Stores an opened document. Analysis happens later, off the protocol thread.
    pub fn open_document(&mut self, mut document: VersionLensTextDocument) {
        if document.workspace_root.is_none() {
            document.workspace_root = self.workspace_root(&document.uri);
        } else {
            document.workspace_root = normalized_workspace_root(document.workspace_root);
        }
        let Some(generation) = self.generation.checked_add(1) else {
            self.documents.clear();
            self.revisions.clear();
            self.cancel_pending_resolutions();
            return;
        };
        self.generation = generation;
        let path = normalized_file_path(&document.uri);
        let core_uri = path
            .as_deref()
            .and_then(versionlens_engine::workspace_file_uri)
            .unwrap_or_else(|| document.uri.clone());
        self.revisions.insert(
            document.uri.clone(),
            DocumentRevision {
                generation: self.generation,
                version: None,
                path,
                core_uri,
            },
        );
        self.documents.insert(document.uri.clone(), document);
        self.synchronize_workspace_documents();
    }

    /// Stores the new text of an open document.
    pub fn change_document(&mut self, uri: &str, text: String) {
        let Some(existing) = self.documents.get(uri) else {
            return;
        };
        let document = VersionLensTextDocument {
            uri: existing.uri.clone(),
            language_id: existing.language_id.clone(),
            text,
            workspace_root: existing.workspace_root.clone(),
        };
        self.open_document(document);
    }

    /// Forgets a closed document.
    pub fn close_document(&mut self, uri: &str) {
        self.documents.remove(uri);
        self.revisions.remove(uri);
        self.synchronize_workspace_documents();
    }

    /// Code lenses for an open document; empty when it is not open.
    pub fn code_lenses(&self, uri: &str) -> Vec<CodeLens> {
        self.analyzed_document(uri)
            .map_or_else(Vec::new, |resolved| resolved.code_lenses)
    }

    /// Diagnostics parameters for `uri`, without a document version.
    pub fn publish_diagnostics(uri: Uri, diagnostics: Vec<Diagnostic>) -> PublishDiagnosticsParams {
        PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        }
    }

    pub(crate) fn analyzed_document(&self, uri: &str) -> Option<ResolvedDocument> {
        let work = self.document_work(uri)?;
        let analysis = self.session_for(uri).analyze_document(work.input);
        let dependency_ranges = &analysis.dependencies;
        Some(ResolvedDocument {
            code_lenses: analysis
                .code_lenses
                .into_iter()
                .flat_map(|payload| into_lsp_code_lenses(payload, dependency_ranges, uri))
                .collect(),
            diagnostics: analysis
                .diagnostics
                .into_iter()
                .map(into_lsp_diagnostic)
                .collect(),
        })
    }

    pub(crate) fn checked_document(&self, input: &DocumentInput) -> Option<CheckedDocument> {
        let event_path = versionlens_engine::workspace_path(&input.uri);
        let open_uri = self.documents.keys().find(|uri| {
            if uri.as_str() == input.uri {
                return true;
            }
            let Some(event_path) = event_path.as_deref() else {
                return false;
            };
            self.revisions
                .get(uri.as_str())
                .and_then(|revision| revision.path.as_deref())
                == Some(event_path)
        });
        let Some(open_uri) = open_uri else {
            // Only editor text carries a version. With no open document left
            // to own it, the result describes text that was closed.
            if input.version.is_some() {
                return None;
            }
            return Some(CheckedDocument {
                uri: input.uri.clone(),
                version: None,
                input: input.clone(),
                open: false,
            });
        };
        let work = self.document_work(open_uri)?;
        if work.input.version != input.version
            || document_text_hash(&work.input.text) != document_text_hash(&input.text)
        {
            return None;
        }
        Some(CheckedDocument {
            uri: open_uri.clone(),
            version: work.version,
            input: input.clone(),
            open: true,
        })
    }

    pub(crate) fn accepts_document_version(&self, uri: &str, version: i32) -> bool {
        self.revisions
            .get(uri)
            .is_some_and(|revision| revision.version.is_none_or(|previous| version > previous))
    }

    pub(crate) fn set_document_version(&mut self, uri: &str, version: i32) {
        if let Some(revision) = self.revisions.get_mut(uri) {
            revision.version = Some(version);
        }
        self.synchronize_workspace_documents();
    }

    /// The work of an open document, unless an ignore file, `files.exclude`,
    /// or `versionlens.workspace.exclude` matches it. Every check, lens,
    /// action, and command reads open documents through here.
    pub(crate) fn document_work(&self, uri: &str) -> Option<DocumentWork> {
        self.open_work(uri)
            .filter(|work| !self.skipped(&work.input.uri))
    }

    fn skipped(&self, uri: &str) -> bool {
        let Some(path) = versionlens_engine::workspace_path(uri) else {
            return false;
        };
        let group = self.group_of(uri);
        // The folders apply even when only open files are checked; the
        // discovery is never iterated, so nothing is scanned.
        let mut scope = self.workspace_scope(group);
        scope.exclusions = self.groups()[group].settings.document_exclusions();
        self.groups()[group]
            .session
            .discover_workspace_documents(scope)
            .skips_file(&path)
    }

    fn open_work(&self, uri: &str) -> Option<DocumentWork> {
        let uri = self
            .documents
            .contains_key(uri)
            .then_some(uri)
            .or_else(|| {
                self.revisions
                    .iter()
                    .find(|(_, revision)| revision.core_uri == uri)
                    .map(|(uri, _)| uri.as_str())
            })?;
        let document = self.documents.get(uri)?;
        let revision = self.revisions.get(uri)?;
        let mut input = document_input(document, &revision.core_uri);
        input.version = revision.version.and_then(|version| version.try_into().ok());
        Some(DocumentWork {
            uri: uri.to_owned(),
            input,
            generation: revision.generation,
            version: revision.version,
        })
    }

    pub(crate) fn work_is_current(&self, work: &DocumentWork) -> bool {
        self.revisions
            .get(&work.uri)
            .is_some_and(|revision| revision.generation == work.generation)
    }
}

/// Maps an engine analysis to LSP lenses and diagnostics. It does no I/O.
pub(crate) fn resolved_document(analysis: AnalyzeDocumentOutput, uri: &str) -> ResolvedDocument {
    let dependency_ranges = &analysis.dependencies;
    ResolvedDocument {
        code_lenses: analysis
            .code_lenses
            .into_iter()
            .flat_map(|payload| into_lsp_code_lenses(payload, dependency_ranges, uri))
            .collect(),
        diagnostics: analysis
            .diagnostics
            .into_iter()
            .map(into_lsp_diagnostic)
            .collect(),
    }
}

fn normalized_workspace_root(root: Option<String>) -> Option<String> {
    let root = root?;
    let normalized = versionlens_engine::workspace_path(&root)
        .and_then(|path| path.canonicalize().ok())
        .and_then(|path| versionlens_engine::workspace_file_uri(&path));
    Some(normalized.unwrap_or(root))
}

fn document_input(document: &VersionLensTextDocument, core_uri: &str) -> DocumentInput {
    DocumentInput::new(
        core_uri.to_owned(),
        document.language_id.clone(),
        document.text.clone(),
        document.workspace_root.clone(),
    )
}

impl VersionLensLspState {
    pub(crate) fn workspace_edit(&self, plan: &WorkspaceEditPlan) -> Result<Value, String> {
        let mut current = Vec::with_capacity(plan.documents.len());
        let mut changes = Vec::with_capacity(plan.documents.len());
        for document in &plan.documents {
            let uri = &document.document.uri;
            let (edit_uri, version, model_version, text) = if let Some(work) = self.open_work(uri) {
                (work.uri, work.version, work.input.version, work.input.text)
            } else {
                let path =
                    versionlens_engine::workspace_path(uri).ok_or("unsupported document URI")?;
                let text = fs::read_to_string(path)
                    .map_err(|error| format!("cannot validate {uri}: {error}"))?;
                (uri.clone(), None, None, text)
            };
            current.push((uri.clone(), model_version, document_text_hash(&text)));
            changes.push(
                json!({"textDocument":{"uri":edit_uri,"version":version},"edits":document.edits}),
            );
        }
        versionlens_engine::validate_workspace_edit_plan(plan, &current).map_err(str::to_owned)?;
        Ok(json!({"documentChanges":changes}))
    }
}
