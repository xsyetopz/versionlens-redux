use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionIndicators {
    pub latest: String,
    pub satisfies_latest: String,
    pub directory: String,
    pub error: String,
    pub no_match: String,
    pub matched: String,
    pub updateable: String,
    pub updateable_vulnerable: String,
    pub build: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SuggestionIndicatorsInput {
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

impl SuggestionIndicators {
    pub fn standard() -> Self {
        Self {
            latest: "\u{1F7E2}".to_owned(),
            satisfies_latest: "\u{1F7E2}".to_owned(),
            directory: "\u{1F4C1}".to_owned(),
            error: "\u{1F534}".to_owned(),
            no_match: "\u{26AA}".to_owned(),
            matched: "\u{1F7E1}".to_owned(),
            updateable: "\u{2191} ".to_owned(),
            updateable_vulnerable: "\u{26A0}\u{FE0F}".to_owned(),
            build: "\u{224C} ".to_owned(),
        }
    }

    pub fn from_input(input: SuggestionIndicatorsInput) -> Self {
        let defaults = Self::standard();
        Self {
            latest: input.latest.unwrap_or(defaults.latest),
            satisfies_latest: input.satisfies_latest.unwrap_or(defaults.satisfies_latest),
            directory: input.directory.unwrap_or(defaults.directory),
            error: input.error.unwrap_or(defaults.error),
            no_match: input.no_match.unwrap_or(defaults.no_match),
            matched: input.matched.unwrap_or(defaults.matched),
            updateable: input.updateable.unwrap_or(defaults.updateable),
            updateable_vulnerable: input
                .updateable_vulnerable
                .unwrap_or(defaults.updateable_vulnerable),
            build: input.build.unwrap_or(defaults.build),
        }
    }
}
