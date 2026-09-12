use napi_derive::napi;
use versionlens_model::DocumentInput;

use super::input::NativeDocumentInput;

#[napi(object)]
pub struct NativeWorkspaceCheckingInput {
    pub roots: Vec<String>,
    pub exclusions: Vec<String>,
    pub provider_exclusions: Option<Vec<NativeWorkspaceProviderExclusion>>,
    pub documents: Vec<NativeDocumentInput>,
}

#[napi(object)]
pub struct NativeWorkspaceProviderExclusion {
    pub ecosystem: String,
    pub patterns: Vec<String>,
}

#[napi(object)]
pub struct NativeWorkspaceGeneration {
    pub generation: String,
}

#[napi(object)]
pub struct NativeWorkspaceCheckEvent {
    pub generation: String,
    pub kind: String,
    pub document: Option<NativeDocumentInput>,
    pub uri: Option<String>,
    pub message: Option<String>,
}

impl From<DocumentInput> for NativeDocumentInput {
    fn from(input: DocumentInput) -> Self {
        Self {
            uri: input.uri,
            language_id: input.language_id,
            text: input.text,
            workspace_root: input.workspace_root,
            version: input.version.and_then(|version| version.try_into().ok()),
        }
    }
}
