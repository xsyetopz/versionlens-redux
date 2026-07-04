use versionlens_parsers::Dependency;
use versionlens_suggestions::{Suggestion, SuggestionStatus};

mod formatters;

use formatters::{
    build_title_text, current_title_text, directory_not_found_title_text, directory_title_text,
    error_title_text, fixed_title_text, invalid_range_title_text, invalid_title_text,
    latest_title_text, no_match_title_text, not_supported_title_text, satisfies_latest_title_text,
    satisfies_title_text, unresolved_title_text, update_title_text,
};

type TitleFormatter = fn(&Dependency, &Suggestion) -> String;

const TITLE_FORMATTERS: &[(SuggestionStatus, TitleFormatter)] = &[
    (SuggestionStatus::BuildAvailable, build_title_text),
    (SuggestionStatus::UpdateAvailable, update_title_text),
    (SuggestionStatus::Current, current_title_text),
    (
        SuggestionStatus::SatisfiesLatest,
        satisfies_latest_title_text,
    ),
    (SuggestionStatus::Satisfies, satisfies_title_text),
    (SuggestionStatus::Directory, directory_title_text),
    (
        SuggestionStatus::DirectoryNotFound,
        directory_not_found_title_text,
    ),
    (SuggestionStatus::Fixed, fixed_title_text),
    (SuggestionStatus::Invalid, invalid_title_text),
    (SuggestionStatus::InvalidRange, invalid_range_title_text),
    (SuggestionStatus::Error, error_title_text),
    (SuggestionStatus::NoMatch, no_match_title_text),
    (SuggestionStatus::NotSupported, not_supported_title_text),
    (SuggestionStatus::Unresolved, unresolved_title_text),
];

pub(super) fn code_lens_title_text(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
) -> String {
    let Some(suggestion) = suggestion else {
        return "Version Lens".to_owned();
    };

    TITLE_FORMATTERS
        .iter()
        .find_map(|(status, formatter)| {
            (*status == suggestion.status).then(|| formatter(dependency, suggestion))
        })
        .unwrap_or_else(|| latest_title_text(dependency, suggestion))
}
