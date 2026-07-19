use serde::{Deserialize, Serialize};
use versionlens_model::Ecosystem;
use versionlens_model::TextEdit;
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
