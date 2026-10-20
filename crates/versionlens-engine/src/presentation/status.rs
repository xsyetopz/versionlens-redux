use versionlens_update::{Suggestion, SuggestionStatus};

use crate::presentation::view::{DiagnosticView, StatusView};

type OptionalSuggestions<'a> = &'a [Option<Suggestion>];

pub(crate) fn status_view(
    dependency_count: usize,
    diagnostics: &[DiagnosticView],
    suggestions: OptionalSuggestions<'_>,
    show_suggestion_stats: bool,
) -> StatusView {
    let dependency_count = to_u32(dependency_count);
    let mut update_count = 0;
    let mut error_count = 0;
    let mut no_match_count = 0;
    for suggestion in suggestions.iter().flatten() {
        match suggestion.status {
            SuggestionStatus::UpdateAvailable => update_count += 1,
            SuggestionStatus::Error
            | SuggestionStatus::DirectoryNotFound
            | SuggestionStatus::Invalid
            | SuggestionStatus::InvalidRange => error_count += 1,
            SuggestionStatus::NoMatch | SuggestionStatus::NotSupported => no_match_count += 1,
            _ => {}
        }
    }
    let update_count = to_u32(update_count);
    let vulnerability_count = to_u32(diagnostics.len());
    let error_count = to_u32(error_count);
    let no_match_count = to_u32(no_match_count);

    let text = if show_suggestion_stats {
        format!(
            "$(versions) {update_count}/{dependency_count} updates, {vulnerability_count} vulnerabilities, {error_count} errors, {no_match_count} no matches"
        )
    } else {
        format!("$(versions) {update_count}/{dependency_count}")
    };

    StatusView {
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

pub(crate) fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
