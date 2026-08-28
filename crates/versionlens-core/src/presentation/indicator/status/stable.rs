use crate::SuggestionIndicators;
use crate::presentation::statuses::{
    StatusBuildAvailable, StatusCurrent, StatusDirectory, StatusDirectoryNotFound, StatusError,
    StatusFixed, StatusInvalid, StatusInvalidRange, StatusNoMatch, StatusNotSupported,
    StatusSatisfies, StatusSatisfiesLatest, StatusUnresolved, StatusUpdateAvailable,
    SuggestionStatus,
};

pub(super) fn stable_indicator(
    indicators: &SuggestionIndicators,
    status: SuggestionStatus,
) -> &str {
    match status {
        StatusCurrent => &indicators.latest,
        StatusSatisfiesLatest => &indicators.satisfies_latest,
        StatusDirectory => &indicators.directory,
        StatusBuildAvailable => &indicators.build,
        StatusFixed | StatusSatisfies => &indicators.matched,
        StatusDirectoryNotFound | StatusError | StatusInvalid | StatusInvalidRange => {
            &indicators.error
        }
        StatusNoMatch | StatusNotSupported | StatusUnresolved => &indicators.no_match,
        StatusUpdateAvailable => &indicators.updateable,
    }
}
