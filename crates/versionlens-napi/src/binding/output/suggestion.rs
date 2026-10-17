use napi_derive::napi;
use versionlens_core::SuggestionView;

use super::dependency::NativeDependency;

#[napi(object)]
pub struct NativeSuggestion {
    pub dependency: NativeDependency,
    pub latest: Option<String>,
    pub status: String,
    pub builds: Vec<String>,
}

impl NativeSuggestion {
    pub(super) fn from_core(suggestion: SuggestionView) -> Self {
        Self {
            dependency: suggestion.dependency.into(),
            latest: suggestion.latest,
            status: suggestion.status,
            builds: suggestion.builds,
        }
    }
}

impl From<SuggestionView> for NativeSuggestion {
    fn from(value: SuggestionView) -> Self {
        Self::from_core(value)
    }
}
