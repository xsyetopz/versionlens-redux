use napi_derive::napi;
use versionlens_core::SuggestionIndicatorsInput;

versionlens_core::define_suggestion_indicators_input!(
    #[napi(object)]
    #[derive(Default)]
    NativeSuggestionIndicators
);

impl NativeSuggestionIndicators {
    pub(in crate::binding::config) fn into_input(self) -> SuggestionIndicatorsInput {
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

impl From<NativeSuggestionIndicators> for SuggestionIndicatorsInput {
    fn from(value: NativeSuggestionIndicators) -> Self {
        value.into_input()
    }
}
