use versionlens_model::Dependency;
use versionlens_suggestions::Suggestion;

pub(in crate::presentation) fn latest_text(suggestion: &Suggestion) -> &str {
    suggestion.latest.as_deref().unwrap_or_default()
}

pub(in crate::presentation::title) fn latest_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    latest_text(suggestion).to_owned()
}
