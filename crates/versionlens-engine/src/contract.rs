use serde::{Deserialize, Serialize};
use std::collections;
use versionlens_model::Ecosystem;
use versionlens_model::{
    DocumentEditPlan, DocumentInput, DocumentSnapshot, TextEdit, WorkspaceEditPlan,
    document_text_hash,
};

use crate::presentation::view::{
    DependencyView, DiagnosticView, LensView, StatusView, SuggestionView,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// What the engine found in one document, without a registry lookup.
pub struct AnalyzeDocumentOutput {
    /// The dependencies in the document.
    pub dependencies: Vec<DependencyView>,
    /// The code lenses to show.
    pub code_lenses: Vec<LensView>,
    /// The diagnostics to show.
    pub diagnostics: Vec<DiagnosticView>,
    /// The status of the document.
    pub status: StatusView,
    /// Whether the dependencies can be sorted.
    pub can_sort_dependencies: bool,
    /// Whether the document is a manifest the engine supports.
    pub is_supported_manifest: bool,
    /// The name of the provider that handles the document, if any.
    pub active_provider_name: Option<String>,
    /// The setting key of the install task for this manifest, if it has one.
    pub install_task_config_key: Option<String>,
    /// A signature of the dependencies, to tell whether they changed.
    pub dependency_signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The result of resolving the versions of one document.
pub struct ResolveDocumentOutput {
    /// One suggestion per dependency.
    pub suggestions: Vec<SuggestionView>,
    /// Text edits that apply the update.
    pub edits: Vec<TextEdit>,
    /// How many registry requests need authorization.
    pub authorization_required_count: u32,
    /// The registry requests that need authorization.
    pub authorization_required_requests: Vec<AuthorizationRequestPayload>,
    /// How many updates are offered for vulnerable versions.
    pub vulnerable_update_count: u32,
    /// The package of the vulnerable update, when there is one.
    pub vulnerable_update_package: Option<String>,
    /// The version of the vulnerable update, when there is one.
    pub vulnerable_update_version: Option<String>,
    /// The edits with the document snapshots they apply to.
    pub edit_plan: Option<WorkspaceEditPlan>,
}

pub(crate) struct ResolveDocumentOutputParts {
    pub(crate) suggestions: Vec<SuggestionView>,
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

/// Checks that `plan` still applies to the documents in `current`.
///
/// `current` holds the URI, version, and text hash of each open document.
///
/// # Errors
///
/// Returns a message when the plan lists a document twice, names a document
/// that is not in `current`, is stale, or has overlapping edits.
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
/// A registry response the editor already fetched.
pub struct RegistryResponseInput {
    /// The package the response is for.
    pub package: String,
    /// The ecosystem of the package.
    pub ecosystem: Ecosystem,
    /// The response body.
    pub body: String,
}

impl RegistryResponseInput {
    /// Creates a response for `package`.
    pub fn new(package: impl Into<String>, ecosystem: Ecosystem, body: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            ecosystem,
            body: body.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use versionlens_model::{Position, Range};

    fn edit(start: u32, end: u32) -> TextEdit {
        TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: start,
                },
                end: Position {
                    line: 0,
                    character: end,
                },
            },
            new_text: String::new(),
        }
    }

    fn plan(edits: Vec<TextEdit>) -> WorkspaceEditPlan {
        WorkspaceEditPlan {
            documents: vec![DocumentEditPlan {
                document: DocumentSnapshot {
                    uri: "file:///a".into(),
                    version: Some(1),
                    text_hash: "hash".into(),
                },
                edits,
            }],
        }
    }

    #[test]
    fn validator_sorts_and_allows_adjacent_edits() {
        assert!(
            validate_workspace_edit_plan(
                &plan(vec![edit(2, 3), edit(0, 1)]),
                &[("file:///a".into(), Some(1), "hash".into())],
            )
            .is_ok()
        );
    }

    #[test]
    fn validator_rejects_overlap_duplicate_and_stale_documents() {
        assert!(
            validate_workspace_edit_plan(
                &plan(vec![edit(0, 2), edit(1, 3)]),
                &[("file:///a".into(), Some(1), "hash".into())],
            )
            .is_err()
        );
        let mut duplicate = plan(vec![]);
        duplicate.documents.push(duplicate.documents[0].clone());
        assert!(
            validate_workspace_edit_plan(
                &duplicate,
                &[("file:///a".into(), Some(1), "hash".into())]
            )
            .is_err()
        );
        assert!(
            validate_workspace_edit_plan(
                &plan(vec![]),
                &[("file:///a".into(), Some(2), "hash".into())]
            )
            .is_err()
        );
    }
}
