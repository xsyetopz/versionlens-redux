use versionlens_suggestions::Suggestion;

use crate::SuggestionIndicators;

mod status;
mod text;

pub(super) use text::with_indicator;

use status::suggestion_indicator;
pub(crate) use status::update::update_indicator;

pub(super) fn code_lens_indicator<'a>(
    indicators: &'a SuggestionIndicators,
    suggestion: Option<&Suggestion>,
    has_vulnerabilities: bool,
) -> &'a str {
    suggestion.map_or(&indicators.no_match, |suggestion| {
        suggestion_indicator(indicators, suggestion.status, has_vulnerabilities)
    })
}
