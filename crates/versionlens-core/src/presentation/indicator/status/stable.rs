use versionlens_suggestions::SuggestionStatus;
use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Current as StatusCurrent, Directory as StatusDirectory,
    DirectoryNotFound as StatusDirectoryNotFound, Error as StatusError, Fixed as StatusFixed,
    Invalid as StatusInvalid, InvalidRange as StatusInvalidRange, NoMatch as StatusNoMatch,
    NotSupported as StatusNotSupported, Satisfies as StatusSatisfies,
    SatisfiesLatest as StatusSatisfiesLatest, Unresolved as StatusUnresolved,
    UpdateAvailable as StatusUpdateAvailable,
};

use crate::SuggestionIndicators;

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
