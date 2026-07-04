use napi_derive::napi;
use versionlens_core::SuggestionIndicatorsInput;

#[napi(object)]
#[derive(Default)]
pub struct NativeSuggestionIndicators {
    pub latest: Option<String>,
    pub satisfies_latest: Option<String>,
    pub directory: Option<String>,
    pub error: Option<String>,
    pub no_match: Option<String>,
    pub matched: Option<String>,
    pub updateable: Option<String>,
    pub updateable_vulnerable: Option<String>,
    pub build: Option<String>,
}

impl NativeSuggestionIndicators {
    pub(in crate::model::config) fn into_input(self) -> SuggestionIndicatorsInput {
        SuggestionIndicatorsInput {
            latest: self.latest,
            satisfies_latest: self.satisfies_latest,
            directory: self.directory,
            error: self.error,
            no_match: self.no_match,
            matched: self.matched,
            updateable: self.updateable,
            updateable_vulnerable: self.updateable_vulnerable,
            build: self.build,
        }
    }
}
