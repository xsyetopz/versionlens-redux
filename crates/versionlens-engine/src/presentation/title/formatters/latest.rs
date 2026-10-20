use versionlens_model::Dependency;
use versionlens_model::Ecosystem::GitHub;
use versionlens_update::Suggestion;

pub(in crate::presentation) fn latest_text(suggestion: &Suggestion) -> &str {
    suggestion.latest.as_deref().unwrap_or_default()
}

pub(in crate::presentation::title) fn latest_title_text(
    dependency: &Dependency,
    suggestion: &Suggestion,
) -> String {
    prefixed_version(dependency, latest_text(suggestion))
}

/// A GitHub version shown with the requirement prefix it lacks, such as `v`.
pub(in crate::presentation) fn prefixed_version(dependency: &Dependency, version: &str) -> String {
    if dependency.ecosystem == GitHub
        && !dependency.requirement_prefix.is_empty()
        && !version.starts_with(&dependency.requirement_prefix)
    {
        return format!("{}{}", dependency.requirement_prefix, version);
    }
    version.to_owned()
}
