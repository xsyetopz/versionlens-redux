use crate::Suggestion;
use crate::SuggestionStatus::{
    InvalidRange as StatusInvalidRange, Satisfies as StatusSatisfies,
    SatisfiesLatest as StatusSatisfiesLatest, UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_model::{CanonicalReference, TextEdit};

use crate::replacement::replacement_text;

mod plan;
pub use plan::{
    Bump, BumpKind, Plan, PlanStatus, floor_crosses_minor, floor_reaches, has_a_floor, plan,
};

pub fn update_edits(suggestions: &[Suggestion]) -> Vec<TextEdit> {
    suggestions
        .iter()
        .filter(|suggestion| update_edit_allowed(suggestion))
        .filter_map(suggestion_update_edit)
        .collect()
}

pub fn bulk_update_edits(suggestions: &[Suggestion]) -> Vec<TextEdit> {
    suggestions
        .iter()
        .filter(|suggestion| update_edit_allowed(suggestion))
        .filter(|suggestion| bulk_update_release_allowed(suggestion.latest.as_deref()))
        .filter_map(suggestion_update_edit)
        .collect()
}

/// The edit to the latest offered target of one suggestion.
pub fn update_edit(suggestion: &Suggestion) -> Option<TextEdit> {
    update_edit_allowed(suggestion)
        .then(|| suggestion_update_edit(suggestion))
        .flatten()
}

fn update_edit_allowed(suggestion: &Suggestion) -> bool {
    match suggestion.status {
        StatusUpdateAvailable | StatusInvalidRange | StatusSatisfies => true,
        StatusSatisfiesLatest => has_a_floor(
            &suggestion.dependency.requirement,
            crate::version_dialect(&suggestion.dependency),
        ),
        _ => false,
    }
}

fn bulk_update_release_allowed(latest: Option<&str>) -> bool {
    let Some(latest) = latest else {
        return false;
    };
    crate::parse_semver(latest.trim()).map_or(true, |version| version.pre.is_empty())
}

fn suggestion_update_edit(suggestion: &Suggestion) -> Option<TextEdit> {
    if suggestion.dependency.is_runtime_constraint() {
        return None;
    }
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
        Some(
            CanonicalReference::GitHubActionSha { .. }
                | CanonicalReference::GitHubActionCommit { .. }
        )
    ))
    .then(|| replacement_text(&suggestion.dependency, latest))
}

#[cfg(test)]
mod tests;
