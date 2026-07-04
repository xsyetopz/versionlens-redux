use versionlens_parsers::{Dependency, Ecosystem};

use crate::model::SuggestionStatus;

pub(super) fn github_commit_status_for_dependency(
    dependency: &Dependency,
    latest: &str,
) -> Option<SuggestionStatus> {
    is_github_commit_dependency(dependency)
        .then(|| github_commit_status(&dependency.requirement, latest))
}

fn github_commit_status(current: &str, latest: &str) -> SuggestionStatus {
    if github_commit_matches(current, latest) {
        SuggestionStatus::Current
    } else {
        SuggestionStatus::UpdateAvailable
    }
}

fn is_github_commit_dependency(dependency: &Dependency) -> bool {
    matches!(dependency.ecosystem, Ecosystem::Npm | Ecosystem::Ruby)
        && dependency
            .hosted_url
            .as_deref()
            .is_some_and(|url| url.ends_with("/commits"))
}

fn github_commit_matches(current: &str, latest: &str) -> bool {
    current == latest || latest.len() == 7 && current.starts_with(latest)
}
