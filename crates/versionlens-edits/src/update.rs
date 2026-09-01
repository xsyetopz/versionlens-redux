use versionlens_model::{CanonicalReference, TextEdit};
use versionlens_suggestions::SuggestionStatus::{
    InvalidRange as StatusInvalidRange, Satisfies as StatusSatisfies,
    UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_suggestions::{Suggestion, SuggestionStatus};

use crate::replacement::replacement_text;

pub fn update_edits(suggestions: &[Suggestion]) -> Vec<TextEdit> {
    suggestions
        .iter()
        .filter(|suggestion| update_edit_allowed(suggestion.status))
        .filter_map(suggestion_update_edit)
        .collect()
}

pub fn bulk_update_edits(suggestions: &[Suggestion]) -> Vec<TextEdit> {
    suggestions
        .iter()
        .filter(|suggestion| update_edit_allowed(suggestion.status))
        .filter(|suggestion| bulk_update_release_allowed(suggestion.latest.as_deref()))
        .filter_map(suggestion_update_edit)
        .collect()
}

fn update_edit_allowed(status: SuggestionStatus) -> bool {
    matches!(
        status,
        StatusUpdateAvailable | StatusInvalidRange | StatusSatisfies
    )
}

fn bulk_update_release_allowed(latest: Option<&str>) -> bool {
    let Some(latest) = latest else {
        return false;
    };
    crate::parse_semver(latest.trim()).map_or(true, |version| version.pre.is_empty())
}

fn suggestion_update_edit(suggestion: &Suggestion) -> Option<TextEdit> {
    let latest = suggestion.latest.as_deref()?;
    Some(TextEdit {
        range: suggestion.dependency.requirement_range,
        new_text: selected_replacement(suggestion, latest)?,
    })
}

fn selected_replacement(suggestion: &Suggestion, latest: &str) -> Option<String> {
    if let Some(replacement) = suggestion
        .choices
        .iter()
        .find(|choice| choice.version == latest)
        .and_then(|choice| choice.replacement.as_deref())
    {
        return Some(replacement.to_owned());
    }
    (!matches!(
        suggestion.dependency.canonical_reference,
        Some(CanonicalReference::GitHubActionSha { .. })
    ))
    .then(|| replacement_text(&suggestion.dependency, latest))
}

#[cfg(test)]
mod tests;
