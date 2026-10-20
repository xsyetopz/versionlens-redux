use versionlens_model::Dependency;
use versionlens_model::Ecosystem::GitHub;
use versionlens_update::{Suggestion, SuggestionStatus};

pub(super) fn code_lens_title_text(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
) -> String {
    let Some(suggestion) = suggestion else {
        return "Version Lens".to_owned();
    };

    let latest = suggestion.latest.as_deref().unwrap_or_default();
    let latest_version = || prefixed_version(dependency, latest);
    match suggestion.status {
        SuggestionStatus::BuildAvailable => "change build".to_owned(),
        SuggestionStatus::UpdateAvailable => title_with_version("latest", &latest_version()),
        SuggestionStatus::Current => {
            let label = if latest.contains('-') {
                "latest prerelease"
            } else {
                "latest"
            };
            title_with_version(label, &latest_version())
        }
        SuggestionStatus::SatisfiesLatest => {
            title_with_version("satisfies latest", &latest_version())
        }
        SuggestionStatus::Satisfies => title_with_version("satisfies", &latest_version()),
        SuggestionStatus::Directory => format!("file://{latest}"),
        SuggestionStatus::DirectoryNotFound => title_with_version("not found", latest),
        SuggestionStatus::Fixed => title_with_version("fixed", &latest_version()),
        SuggestionStatus::Invalid => "invalid version".to_owned(),
        SuggestionStatus::InvalidRange => "invalid version range".to_owned(),
        SuggestionStatus::Error | SuggestionStatus::RateLimited => latest.to_owned(),
        SuggestionStatus::NoMatch => suggestion
            .latest
            .as_deref()
            .unwrap_or("no match")
            .to_owned(),
        SuggestionStatus::NotSupported => "not supported".to_owned(),
        SuggestionStatus::Unresolved => format!("{} unresolved", dependency.name),
    }
}

/// A GitHub version shown with the requirement prefix it lacks, such as `v`.
pub(super) fn prefixed_version(dependency: &Dependency, version: &str) -> String {
    if dependency.ecosystem == GitHub
        && !dependency.requirement_prefix.is_empty()
        && !version.starts_with(&dependency.requirement_prefix)
    {
        return format!("{}{}", dependency.requirement_prefix, version);
    }
    version.to_owned()
}

fn title_with_version(label: &str, version: &str) -> String {
    if version.is_empty() {
        return label.to_owned();
    }
    format!("{label} {version}")
}
