use serde::{Deserialize, Serialize};
use std::collections;
use versionlens_model::Ecosystem;
use versionlens_model::{
    DocumentEditPlan, DocumentInput, DocumentSnapshot, TextEdit, WorkspaceEditPlan,
    document_text_hash,
};
use versionlens_vscode_model::{
    CodeLensPayload, DependencyPayload, DiagnosticPayload, StatusPayload, SuggestionPayload,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeDocumentOutput {
    pub dependencies: Vec<DependencyPayload>,
    pub code_lenses: Vec<CodeLensPayload>,
    pub diagnostics: Vec<DiagnosticPayload>,
    pub status: StatusPayload,
    pub can_sort_dependencies: bool,
    pub is_supported_manifest: bool,
    pub active_provider_name: Option<String>,
    pub install_task_config_key: Option<String>,
    pub dependency_signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveDocumentOutput {
    pub suggestions: Vec<SuggestionPayload>,
    pub edits: Vec<TextEdit>,
    pub authorization_required_count: u32,
    pub authorization_required_requests: Vec<AuthorizationRequestPayload>,
    pub vulnerable_update_count: u32,
    pub vulnerable_update_package: Option<String>,
    pub vulnerable_update_version: Option<String>,
    pub edit_plan: Option<WorkspaceEditPlan>,
}

pub(crate) struct ResolveDocumentOutputParts {
    pub(crate) suggestions: Vec<SuggestionPayload>,
    pub(crate) edits: Vec<TextEdit>,
    pub(crate) authorization_required_count: u32,
    pub(crate) authorization_required_requests: Vec<AuthorizationRequestPayload>,
    pub(crate) vulnerable_update_count: u32,
    pub(crate) vulnerable_update_package: Option<String>,
    pub(crate) vulnerable_update_version: Option<String>,
    pub(crate) edit_plan: Option<WorkspaceEditPlan>,
}

pub(crate) fn resolve_document_output(parts: ResolveDocumentOutputParts) -> ResolveDocumentOutput {
    ResolveDocumentOutput {
        suggestions: parts.suggestions,
        edits: parts.edits,
        authorization_required_count: parts.authorization_required_count,
        authorization_required_requests: parts.authorization_required_requests,
        vulnerable_update_count: parts.vulnerable_update_count,
        vulnerable_update_package: parts.vulnerable_update_package,
        vulnerable_update_version: parts.vulnerable_update_version,
        edit_plan: parts.edit_plan,
    }
}

pub(crate) fn document_edit_plan(
    input: &DocumentInput,
    edits: &[TextEdit],
) -> Option<WorkspaceEditPlan> {
    let mut ordered = edits.to_vec();
    ordered.sort_by_key(|edit| (edit.range.start.line, edit.range.start.character));
    if ordered.windows(2).any(|pair| {
        let left = &pair[0].range;
        let right = &pair[1].range;
        left.end.line > right.start.line
            || (left.end.line == right.start.line && left.end.character > right.start.character)
    }) {
        return None;
    }
    let plan = WorkspaceEditPlan {
        documents: vec![DocumentEditPlan {
            document: DocumentSnapshot {
                uri: input.uri.clone(),
                version: input.version,
                text_hash: document_text_hash(&input.text),
            },
            edits: ordered,
        }],
    };
    let current = vec![(
        input.uri.clone(),
        input.version,
        document_text_hash(&input.text),
    )];
    validate_workspace_edit_plan(&plan, &current)
        .ok()
        .map(|()| plan)
}

pub fn validate_workspace_edit_plan(
    plan: &WorkspaceEditPlan,
    current: &[(String, Option<u64>, String)],
) -> Result<(), &'static str> {
    let mut seen = collections::BTreeSet::new();
    for document in &plan.documents {
        if !seen.insert(document.document.uri.clone()) {
            return Err("duplicate document in edit plan");
        }
        let Some((_, version, hash)) = current
            .iter()
            .find(|(uri, _, _)| uri == &document.document.uri)
        else {
            return Err("document missing from workspace");
        };
        if document.document.version.is_some() && document.document.version != *version
            || document.document.text_hash != *hash
        {
            return Err("stale workspace edit plan");
        }
        let mut edits = document.edits.clone();
        edits.sort_by_key(|edit| (edit.range.start.line, edit.range.start.character));
        if edits.windows(2).any(|pair| {
            pair[0].range.end.line > pair[1].range.start.line
                || (pair[0].range.end.line == pair[1].range.start.line
                    && pair[0].range.end.character > pair[1].range.start.character)
        }) {
            return Err("overlapping edits");
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationRequestPayload {
    pub auth_url: String,
    pub request_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryResponseInput {
    pub package: String,
    pub ecosystem: Ecosystem,
    pub body: String,
}

impl RegistryResponseInput {
    pub fn new(package: impl Into<String>, ecosystem: Ecosystem, body: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            ecosystem,
            body: body.into(),
        }
    }
}

#[cfg(test)]
mod tests;
