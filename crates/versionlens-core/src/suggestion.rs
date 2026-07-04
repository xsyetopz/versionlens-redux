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
        SuggestionStatus::BuildAvailable => "buildAvailable",
        SuggestionStatus::Current => "current",
        SuggestionStatus::Directory => "directory",
        SuggestionStatus::DirectoryNotFound => "directoryNotFound",
        SuggestionStatus::Error => "error",
        SuggestionStatus::Fixed => "fixed",
        SuggestionStatus::Invalid => "invalid",
        SuggestionStatus::InvalidRange => "invalidRange",
        SuggestionStatus::NoMatch => "noMatch",
        SuggestionStatus::NotSupported => "notSupported",
        SuggestionStatus::Satisfies => "satisfies",
        SuggestionStatus::SatisfiesLatest => "satisfiesLatest",
        SuggestionStatus::Unresolved => "unresolved",
        SuggestionStatus::UpdateAvailable => "updateAvailable",
    }
}
