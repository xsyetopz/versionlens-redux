use versionlens_parsers::Dependency;
use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Current as StatusCurrent, Directory as StatusDirectory,
    DirectoryNotFound as StatusDirectoryNotFound, Error as StatusError, Fixed as StatusFixed,
    Invalid as StatusInvalid, InvalidRange as StatusInvalidRange, NoMatch as StatusNoMatch,
    NotSupported as StatusNotSupported, Satisfies as StatusSatisfies,
    SatisfiesLatest as StatusSatisfiesLatest, Unresolved as StatusUnresolved,
    UpdateAvailable as StatusUpdateAvailable,
};
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
    (StatusBuildAvailable, build_title_text),
    (StatusUpdateAvailable, update_title_text),
    (StatusCurrent, current_title_text),
    (StatusSatisfiesLatest, satisfies_latest_title_text),
    (StatusSatisfies, satisfies_title_text),
    (StatusDirectory, directory_title_text),
    (StatusDirectoryNotFound, directory_not_found_title_text),
    (StatusFixed, fixed_title_text),
    (StatusInvalid, invalid_title_text),
    (StatusInvalidRange, invalid_range_title_text),
    (StatusError, error_title_text),
    (StatusNoMatch, no_match_title_text),
    (StatusNotSupported, not_supported_title_text),
    (StatusUnresolved, unresolved_title_text),
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
