use lsp_types::{CodeLens, Diagnostic, PublishDiagnosticsParams, Uri};
use versionlens_model::{DocumentInput, document_text_hash};

use super::presentation::{into_lsp_code_lenses, into_lsp_diagnostic};
use super::{
    CheckedDocument, DocumentRevision, DocumentWork, ResolvedDocument, VersionLensLspState,
    VersionLensTextDocument, normalized_file_path,
};

impl VersionLensLspState {
    pub fn open_document(&mut self, mut document: VersionLensTextDocument) -> Vec<Diagnostic> {
        if document.workspace_root.is_none() {
            document.workspace_root = self.workspace_root(&document.uri);
        } else {
            document.workspace_root = normalized_workspace_root(document.workspace_root);
        }
        let Some(generation) = self.generation.checked_add(1) else {
            self.documents.clear();
            self.revisions.clear();
            self.session.cancel_pending_resolutions();
            return Vec::new();
        };
        self.generation = generation;
        let path = normalized_file_path(&document.uri);
        let core_uri = path
            .as_deref()
            .and_then(versionlens_core::workspace_file_uri)
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
        let uri = document.uri.clone();
        self.documents.insert(uri.clone(), document);
        self.synchronize_workspace_documents();
        self.analyzed_document(&uri)
            .map_or_else(Vec::new, |document| document.diagnostics)
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
        self.revisions.remove(uri);
        self.synchronize_workspace_documents();
    }

    pub fn code_lenses(&self, uri: &str) -> Vec<CodeLens> {
        if let Some(work) = self.document_work(uri)
            && !self.session.document_is_fresh(&work.input)
        {
            self.session.resolve_document(work.input);
        }
        self.analyzed_document(uri)
            .map_or_else(Vec::new, |resolved| resolved.code_lenses)
    }

    pub fn publish_diagnostics(uri: Uri, diagnostics: Vec<Diagnostic>) -> PublishDiagnosticsParams {
        PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        }
    }

    pub(crate) fn analyzed_document(&self, uri: &str) -> Option<ResolvedDocument> {
        let document = self.documents.get(uri)?;
        let analysis = self.analyze_document(document);
        let dependency_ranges = &analysis.dependencies;
        Some(ResolvedDocument {
            code_lenses: analysis
                .code_lenses
                .into_iter()
                .flat_map(|payload| {
                    into_lsp_code_lenses(
                        payload,
                        dependency_ranges,
                        uri,
                        self.revisions
                            .get(uri)
                            .map_or(0, |revision| revision.generation),
                    )
                })
                .collect(),
            diagnostics: analysis
                .diagnostics
                .into_iter()
                .map(into_lsp_diagnostic)
                .collect(),
        })
    }

    pub(crate) fn analyzed_input(
        &self,
        input: DocumentInput,
        uri: &str,
        generation: u64,
    ) -> ResolvedDocument {
        let analysis = self.session.analyze_document(input);
        let dependency_ranges = &analysis.dependencies;
        ResolvedDocument {
            code_lenses: analysis
                .code_lenses
                .into_iter()
                .flat_map(|payload| {
                    into_lsp_code_lenses(payload, dependency_ranges, uri, generation)
                })
                .collect(),
            diagnostics: analysis
                .diagnostics
                .into_iter()
                .map(into_lsp_diagnostic)
                .collect(),
        }
    }

    pub(crate) fn checked_document(&self, input: &DocumentInput) -> Option<CheckedDocument> {
        let event_path = versionlens_core::workspace_path(&input.uri);
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
            return Some(CheckedDocument {
                uri: input.uri.clone(),
                version: None,
                generation: 0,
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
            generation: work.generation,
            input: input.clone(),
            open: true,
        })
    }

    pub(crate) fn accepts_document_version(&self, uri: &str, version: i32) -> bool {
        self.revisions
            .get(uri)
            .is_some_and(|revision| revision.version.is_none_or(|previous| version > previous))
    }

    pub(crate) fn document_is_open(&self, uri: &str) -> bool {
        self.documents.contains_key(uri)
    }

    pub(crate) fn diagnostic_target(&self, uri: &str) -> (String, Option<i32>) {
        let event_path = versionlens_core::workspace_path(uri);
        if let Some(root) = self
            .workspace_folders
            .iter()
            .chain(self.root_uri.iter())
            .find(|root| root.path.as_deref().map(std::path::Path::new) == event_path.as_deref())
        {
            return (root.uri.to_string(), None);
        }
        self.documents
            .keys()
            .find(|open_uri| {
                open_uri.as_str() == uri
                    || event_path.as_deref().is_some_and(|event_path| {
                        self.revisions
                            .get(open_uri.as_str())
                            .and_then(|revision| revision.path.as_deref())
                            == Some(event_path)
                    })
            })
            .map_or_else(
                || (uri.to_owned(), None),
                |open_uri| {
                    (
                        open_uri.clone(),
                        self.revisions
                            .get(open_uri.as_str())
                            .and_then(|revision| revision.version),
                    )
                },
            )
    }

    pub(crate) fn set_document_version(&mut self, uri: &str, version: i32) {
        if let Some(revision) = self.revisions.get_mut(uri) {
            revision.version = Some(version);
        }
        self.synchronize_workspace_documents();
    }

    pub(crate) fn document_work(&self, uri: &str) -> Option<DocumentWork> {
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

    fn analyze_document(
        &self,
        document: &VersionLensTextDocument,
    ) -> versionlens_core::AnalyzeDocumentOutput {
        let input = self.document_work(&document.uri).map_or_else(
            || document_input(document, &document.uri),
            |work| work.input,
        );
        self.session.analyze_document(input)
    }
}

fn normalized_workspace_root(root: Option<String>) -> Option<String> {
    let root = root?;
    let normalized = versionlens_core::workspace_path(&root)
        .and_then(|path| path.canonicalize().ok())
        .and_then(|path| versionlens_core::workspace_file_uri(&path));
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
