use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, atomic::Ordering};

use versionlens_model::{DocumentInput, ManifestKind, document_text_hash};

use crate::VersionLensSession;
use crate::registry::{
    RegistryContext, RegistryFileSnapshot, registry_context_from_document_kind_with_files,
};
use crate::workspace::{
    WorkspaceDocuments, WorkspaceGraph, document_path, workspace_path, workspace_root,
};

const MAX_REGISTRY_CONTEXTS: usize = 256;

#[derive(Debug, Default)]
pub(crate) struct WorkspaceState {
    generation: u64,
    documents: Arc<WorkspaceDocuments>,
    registry_documents: Arc<WorkspaceDocuments>,
    graphs: BTreeMap<PathBuf, WorkspaceGraph>,
    registry_files: RegistryFileSnapshot,
    registry_contexts: BTreeMap<RegistryContextKey, RegistryContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RegistryContextKey {
    uri: String,
    workspace_root: Option<String>,
    manifest_kind: String,
    text_hash: String,
}

impl RegistryContextKey {
    fn new(input: &DocumentInput, kind: ManifestKind) -> Self {
        Self {
            uri: input.uri.clone(),
            workspace_root: input.workspace_root.clone(),
            manifest_kind: format!("{kind:?}"),
            text_hash: document_text_hash(&input.text),
        }
    }
}

impl VersionLensSession {
    /// Replaces the complete set of unsaved workspace documents.
    ///
    /// Invalid URIs and documents outside their declared workspace root do not
    /// participate in workspace discovery.
    pub fn set_workspace_documents(&self, documents: Vec<DocumentInput>) -> bool {
        let mut workspace_documents = WorkspaceDocuments::new();
        let mut registry_documents = WorkspaceDocuments::new();
        for document in documents {
            let workspace_document_path = document_path(&document);
            let registry_document_path = workspace_document_path.clone().or_else(|| {
                document
                    .workspace_root
                    .is_none()
                    .then(|| workspace_path(&document.uri))
                    .flatten()
            });
            if let Some(path) = workspace_document_path {
                workspace_documents.insert(path, (document.text.clone(), document.version));
            }
            if let Some(path) = registry_document_path {
                registry_documents.insert(path, (document.text, document.version));
            }
        }
        let mut state = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(crate::recover_poison);
        if state.documents.as_ref() == &workspace_documents
            && state.registry_documents.as_ref() == &registry_documents
        {
            return false;
        }
        state.documents = Arc::new(workspace_documents);
        state.registry_documents = Arc::new(registry_documents);
        invalidate_state(self, &mut state);
        self.suggestion_cache().clear();
        drop(state);
        true
    }

    /// Signals that workspace files or workspace configuration changed on disk.
    pub fn invalidate_workspace(&self) {
        let mut state = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(crate::recover_poison);
        invalidate_state(self, &mut state);
        self.suggestion_cache().clear();
        drop(state);
    }

    pub(crate) fn clear_workspace_graphs(&self) {
        let mut state = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(crate::recover_poison);
        state.generation = state.generation.wrapping_add(1);
        state.graphs.clear();
        state.registry_files = RegistryFileSnapshot::new(state.registry_documents.clone());
        state.registry_contexts.clear();
    }

    pub(crate) fn workspace_documents(&self) -> Arc<WorkspaceDocuments> {
        self.storage_state
            .workspace
            .lock()
            .unwrap_or_else(crate::recover_poison)
            .documents
            .clone()
    }

    pub(crate) fn workspace_graph(&self, input: &DocumentInput) -> WorkspaceGraph {
        let Some(root) = workspace_root(input) else {
            return WorkspaceGraph::default();
        };
        let (generation, documents, cached) = {
            let state = self
                .storage_state
                .workspace
                .lock()
                .unwrap_or_else(crate::recover_poison);
            (
                state.generation,
                state.documents.clone(),
                state.graphs.get(&root).cloned(),
            )
        };

        let graph = if let Some(graph) = cached {
            graph
        } else {
            let graph = WorkspaceGraph::for_workspace(&root, &documents);
            let mut state = self
                .storage_state
                .workspace
                .lock()
                .unwrap_or_else(crate::recover_poison);
            let graph = if state.generation == generation {
                state.graphs.entry(root.clone()).or_insert(graph).clone()
            } else {
                graph
            };
            drop(state);
            graph
        };
        if graph.captures(input) || !workspace_input_affects_graph(input) {
            return graph.bind_source(input);
        }

        let mut documents = documents.as_ref().clone();
        if let Some(current) = document_path(input) {
            documents.insert(current, (input.text.clone(), input.version));
        }
        WorkspaceGraph::for_workspace(&root, &documents).bind_source(input)
    }

    pub(crate) fn registry_context(
        &self,
        input: &DocumentInput,
        kind: ManifestKind,
    ) -> RegistryContext {
        let key = RegistryContextKey::new(input, kind);
        let (generation, files) = {
            let state = self
                .storage_state
                .workspace
                .lock()
                .unwrap_or_else(crate::recover_poison);
            if let Some(context) = state.registry_contexts.get(&key) {
                return context.clone();
            }
            (state.generation, state.registry_files.clone())
        };
        let reader = files.reader(input);
        let context = registry_context_from_document_kind_with_files(input, kind, &reader);
        let mut state = self
            .storage_state
            .workspace
            .lock()
            .unwrap_or_else(crate::recover_poison);
        if state.generation != generation {
            return context;
        }
        if state.registry_contexts.len() >= MAX_REGISTRY_CONTEXTS
            && !state.registry_contexts.contains_key(&key)
        {
            state.registry_contexts.pop_first();
        }
        state
            .registry_contexts
            .entry(key)
            .or_insert(context)
            .clone()
    }
}

fn workspace_input_affects_graph(input: &DocumentInput) -> bool {
    document_path(input).is_some_and(|path| {
        matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("package.json" | "Cargo.toml" | "pnpm-workspace.yaml" | "lerna.json")
        )
    })
}

fn invalidate_state(session: &VersionLensSession, state: &mut WorkspaceState) {
    state.generation = state.generation.wrapping_add(1);
    state.graphs.clear();
    state.registry_files = RegistryFileSnapshot::new(state.registry_documents.clone());
    state.registry_contexts.clear();
    session
        .storage_state
        .cache_epoch
        .fetch_add(1, Ordering::AcqRel);
}

#[cfg(test)]
mod tests;
