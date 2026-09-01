use versionlens_model::Dependency;
use versionlens_model::Ecosystem::GitHub;
use versionlens_suggestions::Suggestion;

pub(in crate::presentation) fn latest_text(suggestion: &Suggestion) -> &str {
    suggestion.latest.as_deref().unwrap_or_default()
}

pub(in crate::presentation::title) fn latest_title_text(
    dependency: &Dependency,
    suggestion: &Suggestion,
) -> String {
    let latest = latest_text(suggestion);
    if dependency.ecosystem == GitHub
        && !dependency.requirement_prefix.is_empty()
        && !latest.starts_with(&dependency.requirement_prefix)
    {
        return format!("{}{}", dependency.requirement_prefix, latest);
    }
    latest.to_owned()
}
