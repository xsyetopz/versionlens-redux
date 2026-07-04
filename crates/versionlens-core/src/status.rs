use versionlens_suggestions::{Suggestion, SuggestionStatus};
use versionlens_vscode_model::{DiagnosticPayload, StatusPayload};

pub(crate) fn status_payload(
    dependency_count: usize,
    diagnostics: &[DiagnosticPayload],
    suggestions: &[Option<Suggestion>],
    show_suggestion_stats: bool,
) -> StatusPayload {
    let dependency_count = to_u32(dependency_count);
    let update_count = suggestions_with_status(suggestions, SuggestionStatus::UpdateAvailable);
    let vulnerability_count = diagnostics.len();
    let error_count = suggestions_with_status(suggestions, SuggestionStatus::Error)
        + suggestions_with_status(suggestions, SuggestionStatus::DirectoryNotFound)
        + suggestions_with_status(suggestions, SuggestionStatus::Invalid)
        + suggestions_with_status(suggestions, SuggestionStatus::InvalidRange);
    let no_match_count = suggestions_with_status(suggestions, SuggestionStatus::NoMatch)
        + suggestions_with_status(suggestions, SuggestionStatus::NotSupported);
    let update_count = to_u32(update_count);
    let vulnerability_count = to_u32(vulnerability_count);
    let error_count = to_u32(error_count);
    let no_match_count = to_u32(no_match_count);

    let text = if show_suggestion_stats {
        format!(
            "$(versions) {update_count}/{dependency_count} updates, {vulnerability_count} vulnerabilities, {error_count} errors, {no_match_count} no matches"
        )
    } else {
        format!("$(versions) {update_count}/{dependency_count}")
    };

    StatusPayload {
        dependency_count,
        update_count,
        vulnerability_count,
        error_count,
        no_match_count,
        visible: dependency_count > 0,
        text,
        tooltip: format!(
            "{update_count} updates, {vulnerability_count} vulnerabilities, {error_count} errors, {no_match_count} no matches across {dependency_count} dependencies"
        ),
    }
}

fn suggestions_with_status(suggestions: &[Option<Suggestion>], status: SuggestionStatus) -> usize {
    suggestions
        .iter()
        .filter(|suggestion| {
            suggestion
                .as_ref()
                .is_some_and(|item| item.status == status)
        })
        .count()
}

pub(crate) fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
