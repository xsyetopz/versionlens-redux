use versionlens_suggestions::SuggestionStatus;
use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, UpdateAvailable as StatusUpdateAvailable,
};

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
    if matches!(status, StatusUpdateAvailable | StatusBuildAvailable) && has_vulnerabilities {
        return update_indicator(indicators, true);
    }

    if status == StatusUpdateAvailable {
        return update_indicator(indicators, false);
    }

    stable_indicator(indicators, status)
}
