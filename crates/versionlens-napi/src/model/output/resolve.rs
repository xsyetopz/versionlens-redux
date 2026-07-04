use napi_derive::napi;
use versionlens_core::{AuthorizationRequestPayload, ResolveDocumentOutput};

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
}

impl NativeResolveDocumentOutput {
    pub(crate) fn empty() -> Self {
        Self {
            suggestions: Vec::new(),
            edits: Vec::new(),
            authorization_required_count: 0,
            authorization_required_requests: Vec::new(),
            vulnerable_update_count: 0,
            vulnerable_update_package: None,
            vulnerable_update_version: None,
        }
    }
    pub(crate) fn from_core(output: ResolveDocumentOutput) -> Self {
        Self {
            suggestions: output
                .suggestions
                .into_iter()
                .map(NativeSuggestion::from_core)
                .collect(),
            edits: output
                .edits
                .into_iter()
                .map(NativeTextEdit::from_core)
                .collect(),
            authorization_required_count: output.authorization_required_count,
            authorization_required_requests: output
                .authorization_required_requests
                .into_iter()
                .map(NativeAuthorizationRequest::from_core)
                .collect(),
            vulnerable_update_count: output.vulnerable_update_count,
            vulnerable_update_package: output.vulnerable_update_package,
            vulnerable_update_version: output.vulnerable_update_version,
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
