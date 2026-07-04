use versionlens_suggestions::SuggestionStatus;

use crate::SuggestionIndicators;

mod stable;
pub(crate) mod update;

use stable::stable_indicator;
use update::update_indicator;

pub(super) fn suggestion_indicator(
    indicators: &SuggestionIndicators,
    status: SuggestionStatus,
    has_vulnerabilities: bool,
) -> &str {
    if matches!(
        status,
        SuggestionStatus::UpdateAvailable | SuggestionStatus::BuildAvailable
    ) && has_vulnerabilities
    {
        return update_indicator(indicators, true);
    }

    if status == SuggestionStatus::UpdateAvailable {
        return update_indicator(indicators, false);
    }

    stable_indicator(indicators, status)
}
