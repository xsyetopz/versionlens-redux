use crate::suggestion::SuggestionStatus::{
    Current as StatusCurrent, UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_model::Dependency;

use crate::suggestion::SuggestionStatus;
use versionlens_model::Ecosystem::{Npm, Ruby};

pub(super) fn github_commit_status_for_dependency(
    dependency: &Dependency,
    latest: &str,
) -> Option<SuggestionStatus> {
    is_github_commit_dependency(dependency)
        .then(|| github_commit_status(&dependency.requirement, latest))
}

fn github_commit_status(current: &str, latest: &str) -> SuggestionStatus {
    if github_commit_matches(current, latest) {
        StatusCurrent
    } else {
        StatusUpdateAvailable
    }
}

fn is_github_commit_dependency(dependency: &Dependency) -> bool {
    matches!(dependency.ecosystem, Npm | Ruby)
        && dependency
            .hosted_url
            .as_deref()
            .is_some_and(|url| url.ends_with("/commits"))
}

fn github_commit_matches(current: &str, latest: &str) -> bool {
    current == latest || latest.len() == 7 && current.starts_with(latest)
}
