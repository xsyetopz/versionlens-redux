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

#[macro_export]
macro_rules! define_suggestion_indicators_input {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        pub struct $name {
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
    };
}

define_suggestion_indicators_input!(
    #[derive(Debug, Default, PartialEq, Eq)]
    SuggestionIndicatorsInput
);

impl SuggestionIndicators {
    pub fn standard() -> Self {
        standard_suggestion_indicators()
    }

    pub fn from_input(input: SuggestionIndicatorsInput) -> Self {
        Self {
            latest: input.latest.unwrap_or_default(),
            satisfies_latest: input.satisfies_latest.unwrap_or_default(),
            directory: input.directory.unwrap_or_default(),
            error: input.error.unwrap_or_default(),
            no_match: input.no_match.unwrap_or_default(),
            matched: input.matched.unwrap_or_default(),
            updateable: input.updateable.unwrap_or_default(),
            updateable_vulnerable: input.updateable_vulnerable.unwrap_or_default(),
            build: input.build.unwrap_or_default(),
        }
        .with_standard_indicators_for_blanks()
    }

    pub(crate) fn with_standard_indicators_for_blanks(self) -> Self {
        let defaults = Self::standard();
        Self {
            latest: indicator_or_default(self.latest, defaults.latest),
            satisfies_latest: indicator_or_default(
                self.satisfies_latest,
                defaults.satisfies_latest,
            ),
            directory: indicator_or_default(self.directory, defaults.directory),
            error: indicator_or_default(self.error, defaults.error),
            no_match: indicator_or_default(self.no_match, defaults.no_match),
            matched: indicator_or_default(self.matched, defaults.matched),
            updateable: indicator_or_default(self.updateable, defaults.updateable),
            updateable_vulnerable: indicator_or_default(
                self.updateable_vulnerable,
                defaults.updateable_vulnerable,
            ),
            build: indicator_or_default(self.build, defaults.build),
        }
    }
}

fn indicator_or_default(value: String, default: String) -> String {
    (!value.trim().is_empty())
        .then_some(value)
        .unwrap_or(default)
}

pub fn standard_suggestion_indicators() -> SuggestionIndicators {
    SuggestionIndicators {
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
