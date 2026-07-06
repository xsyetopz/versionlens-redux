use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Current as StatusCurrent, Directory as StatusDirectory,
    DirectoryNotFound as StatusDirectoryNotFound, Error as StatusError, Fixed as StatusFixed,
    Invalid as StatusInvalid, InvalidRange as StatusInvalidRange, NoMatch as StatusNoMatch,
    NotSupported as StatusNotSupported, Satisfies as StatusSatisfies,
    SatisfiesLatest as StatusSatisfiesLatest, Unresolved as StatusUnresolved,
    UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_suggestions::{Suggestion, SuggestionStatus};
use versionlens_vscode_model::SuggestionPayload;

use crate::dependency::dependency_payload;

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
