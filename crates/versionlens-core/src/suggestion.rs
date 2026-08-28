use versionlens_suggestions::Suggestion;
use versionlens_vscode_model::SuggestionPayload;

use crate::dependency::dependency_payload;
use crate::presentation::statuses::{
    StatusBuildAvailable, StatusCurrent, StatusDirectory, StatusDirectoryNotFound, StatusError,
    StatusFixed, StatusInvalid, StatusInvalidRange, StatusNoMatch, StatusNotSupported,
    StatusSatisfies, StatusSatisfiesLatest, StatusUnresolved, StatusUpdateAvailable,
    SuggestionStatus,
};

pub(crate) fn into_suggestion_payloads(suggestions: Vec<Suggestion>) -> Vec<SuggestionPayload> {
    suggestions.into_iter().map(suggestion_payload).collect()
}

fn suggestion_payload(suggestion: Suggestion) -> SuggestionPayload {
    let status = suggestion_status_name(suggestion.status).to_owned();
    SuggestionPayload {
        dependency: dependency_payload(suggestion.dependency),
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
        StatusSatisfies => "satisfies",
        StatusSatisfiesLatest => "satisfiesLatest",
        StatusUnresolved => "unresolved",
        StatusUpdateAvailable => "updateAvailable",
    }
}
