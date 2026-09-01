use napi_derive::napi;
use versionlens_core::{AuthorizationRequestPayload, ResolveDocumentOutput};
use versionlens_model::{DocumentEditPlan, WorkspaceEditPlan};

use super::suggestion::NativeSuggestion;
use super::text_edit::NativeTextEdit;

#[napi(object)]
pub struct NativeResolveDocumentOutput {
    pub suggestions: Vec<NativeSuggestion>,
    pub edits: Vec<NativeTextEdit>,
    pub authorization_required_count: u32,
    pub authorization_required_requests: Vec<NativeAuthorizationRequest>,
    pub vulnerable_update_count: u32,
    pub vulnerable_update_package: Option<String>,
    pub vulnerable_update_version: Option<String>,
    pub edit_plan: Option<NativeWorkspaceEditPlan>,
}

impl NativeResolveDocumentOutput {
    pub(crate) fn empty() -> Self {
        Self {
            suggestions: vec![],
            edits: vec![],
            authorization_required_count: 0,
            authorization_required_requests: vec![],
            vulnerable_update_count: 0,
            vulnerable_update_package: None,
            vulnerable_update_version: None,
            edit_plan: None,
        }
    }
    pub(crate) fn from_core(output: ResolveDocumentOutput) -> Self {
        Self {
            suggestions: output
                .suggestions
                .into_iter()
                .map(|suggestion| suggestion.into())
                .collect(),
            edits: output.edits.into_iter().map(|edit| edit.into()).collect(),
            authorization_required_count: output.authorization_required_count,
            authorization_required_requests: output
                .authorization_required_requests
                .into_iter()
                .map(|request| request.into())
                .collect(),
            vulnerable_update_count: output.vulnerable_update_count,
            vulnerable_update_package: output.vulnerable_update_package,
            vulnerable_update_version: output.vulnerable_update_version,
            edit_plan: output.edit_plan.map(Into::into),
        }
    }
}

#[napi(object)]
pub struct NativeDocumentEditPlan {
    pub document: NativeDocumentSnapshot,
    pub edits: Vec<NativeTextEdit>,
}

#[napi(object)]
pub struct NativeWorkspaceEditPlan {
    pub documents: Vec<NativeDocumentEditPlan>,
}

#[napi(object)]
pub struct NativeDocumentSnapshot {
    pub uri: String,
    pub version: Option<u32>,
    pub text_hash: String,
}

impl From<DocumentEditPlan> for NativeDocumentEditPlan {
    fn from(value: DocumentEditPlan) -> Self {
        Self {
            document: NativeDocumentSnapshot {
                uri: value.document.uri,
                version: value
                    .document
                    .version
                    .and_then(|version| u32::try_from(version).ok()),
                text_hash: value.document.text_hash,
            },
            edits: value.edits.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<WorkspaceEditPlan> for NativeWorkspaceEditPlan {
    fn from(value: WorkspaceEditPlan) -> Self {
        Self {
            documents: value.documents.into_iter().map(Into::into).collect(),
        }
    }
}

#[napi(object)]
pub struct NativeAuthorizationRequest {
    pub auth_url: String,
    pub request_url: String,
}

impl NativeAuthorizationRequest {
    fn from_core(input: AuthorizationRequestPayload) -> Self {
        Self {
            auth_url: input.auth_url,
            request_url: input.request_url,
        }
    }
}

impl Default for NativeResolveDocumentOutput {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<ResolveDocumentOutput> for NativeResolveDocumentOutput {
    fn from(value: ResolveDocumentOutput) -> Self {
        Self::from_core(value)
    }
}

impl From<AuthorizationRequestPayload> for NativeAuthorizationRequest {
    fn from(value: AuthorizationRequestPayload) -> Self {
        Self::from_core(value)
    }
}
