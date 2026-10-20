use versionlens_update::Suggestion;

use crate::SuggestionIndicators;
use crate::presentation::statuses::{
    StatusBuildAvailable, StatusCurrent, StatusDirectory, StatusDirectoryNotFound, StatusError,
    StatusFixed, StatusInvalid, StatusInvalidRange, StatusNoMatch, StatusNotSupported,
    StatusRateLimited, StatusSatisfies, StatusSatisfiesLatest, StatusUnresolved,
    StatusUpdateAvailable, SuggestionStatus,
};

pub(super) fn with_indicator(indicator: &str, title: String) -> String {
    if indicator.is_empty() {
        return title;
    }

    let separator = if cfg!(windows) { "" } else { " " };
    format!("{indicator}{separator}{title}").trim().to_owned()
}

pub(super) fn update_indicator(
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> &str {
    if has_vulnerabilities {
        if indicators.updateable_vulnerable.is_empty() {
            return "⚠️";
        }
        return &indicators.updateable_vulnerable;
    }

    &indicators.updateable
}

pub(super) fn code_lens_indicator<'a>(
    indicators: &'a SuggestionIndicators,
    suggestion: Option<&Suggestion>,
    has_vulnerabilities: bool,
) -> &'a str {
    suggestion.map_or(&indicators.no_match, |suggestion| {
        suggestion_indicator(indicators, suggestion.status, has_vulnerabilities)
    })
}

fn suggestion_indicator(
    indicators: &SuggestionIndicators,
    status: SuggestionStatus,
    has_vulnerabilities: bool,
) -> &str {
    if matches!(status, StatusUpdateAvailable | StatusBuildAvailable) && has_vulnerabilities {
        return update_indicator(indicators, true);
    }

    match status {
        StatusUpdateAvailable => update_indicator(indicators, false),
        StatusCurrent => &indicators.latest,
        StatusSatisfiesLatest => &indicators.satisfies_latest,
        StatusDirectory => &indicators.directory,
        StatusBuildAvailable => &indicators.build,
        StatusFixed | StatusSatisfies => &indicators.matched,
        StatusDirectoryNotFound
        | StatusError
        | StatusRateLimited
        | StatusInvalid
        | StatusInvalidRange => &indicators.error,
        StatusNoMatch | StatusNotSupported | StatusUnresolved => &indicators.no_match,
    }
}
