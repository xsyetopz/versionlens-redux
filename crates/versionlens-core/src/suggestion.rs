use versionlens_update::Suggestion;

use crate::dependency::dependency_view;
use crate::presentation::statuses::{
    StatusBuildAvailable, StatusCurrent, StatusDirectory, StatusDirectoryNotFound, StatusError,
    StatusFixed, StatusInvalid, StatusInvalidRange, StatusNoMatch, StatusNotSupported,
    StatusRateLimited, StatusSatisfies, StatusSatisfiesLatest, StatusUnresolved,
    StatusUpdateAvailable, SuggestionStatus,
};
use crate::view::SuggestionView;

pub(crate) fn into_suggestion_views(suggestions: Vec<Suggestion>) -> Vec<SuggestionView> {
    suggestions.into_iter().map(suggestion_view).collect()
}

fn suggestion_view(suggestion: Suggestion) -> SuggestionView {
    let status = suggestion_status_name(suggestion.status).to_owned();
    SuggestionView {
        dependency: dependency_view(suggestion.dependency),
        latest: suggestion.latest,
        status,
        builds: suggestion.builds,
    }
}

fn suggestion_status_name(status: SuggestionStatus) -> &'static str {
    match status {
        StatusBuildAvailable => "buildAvailable",
        StatusCurrent => "current",
        StatusDirectory => "directory",
        StatusDirectoryNotFound => "directoryNotFound",
        StatusError => "error",
        StatusFixed => "fixed",
        StatusInvalid => "invalid",
        StatusInvalidRange => "invalidRange",
        StatusNoMatch => "noMatch",
        StatusNotSupported => "notSupported",
        StatusRateLimited => "rateLimited",
        StatusSatisfies => "satisfies",
        StatusSatisfiesLatest => "satisfiesLatest",
        StatusUnresolved => "unresolved",
        StatusUpdateAvailable => "updateAvailable",
    }
}
