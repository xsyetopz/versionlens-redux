use versionlens_suggestions::SuggestionStatus;

use crate::SuggestionIndicators;

pub(super) fn stable_indicator(
    indicators: &SuggestionIndicators,
    status: SuggestionStatus,
) -> &str {
    match status {
        SuggestionStatus::Current => &indicators.latest,
        SuggestionStatus::SatisfiesLatest => &indicators.satisfies_latest,
        SuggestionStatus::Directory => &indicators.directory,
        SuggestionStatus::BuildAvailable => &indicators.build,
        SuggestionStatus::Fixed | SuggestionStatus::Satisfies => &indicators.matched,
        SuggestionStatus::DirectoryNotFound
        | SuggestionStatus::Error
        | SuggestionStatus::Invalid
        | SuggestionStatus::InvalidRange => &indicators.error,
        SuggestionStatus::NoMatch
        | SuggestionStatus::NotSupported
        | SuggestionStatus::Unresolved => &indicators.no_match,
        SuggestionStatus::UpdateAvailable => &indicators.updateable,
    }
}
