use versionlens_suggestions::SuggestionStatus::{
    InvalidRange as StatusInvalidRange, Satisfies as StatusSatisfies,
    UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_suggestions::{Suggestion, SuggestionStatus};
use versionlens_vscode_model::TextEdit;

use crate::replacement::replacement_text;

pub fn update_edits(suggestions: &[Suggestion]) -> Vec<TextEdit> {
    suggestions
        .iter()
        .filter(|suggestion| update_edit_allowed(suggestion.status))
        .filter_map(|suggestion| {
            suggestion.latest.as_ref().map(|latest| TextEdit {
                range: suggestion.dependency.requirement_range,
                new_text: replacement_text(&suggestion.dependency, latest),
            })
        })
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
    suggestion.latest.as_ref().map(|latest| TextEdit {
        range: suggestion.dependency.requirement_range,
        new_text: replacement_text(&suggestion.dependency, latest),
    })
}

#[cfg(test)]
mod tests;
