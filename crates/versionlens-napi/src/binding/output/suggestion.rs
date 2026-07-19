use napi_derive::napi;
use versionlens_vscode_model::SuggestionPayload;

use super::dependency::NativeDependency;

#[napi(object)]
pub struct NativeSuggestion {
    pub dependency: NativeDependency,
    pub latest: Option<String>,
    pub status: String,
    pub builds: Vec<String>,
}

impl NativeSuggestion {
    pub(super) fn from_core(suggestion: SuggestionPayload) -> Self {
        Self {
            dependency: suggestion.dependency.into(),
            latest: suggestion.latest,
            status: suggestion.status,
            builds: suggestion.builds,
        }
    }
}

impl From<SuggestionPayload> for NativeSuggestion {
    fn from(value: SuggestionPayload) -> Self {
        Self::from_core(value)
    }
}
