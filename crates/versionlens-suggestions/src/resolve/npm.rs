use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Npm;

pub(super) fn is_npm_dist_tag_dependency(dependency: &Dependency, latest: &str) -> bool {
    let requirement = dependency.requirement.trim();
    dependency.ecosystem == Npm
        && requirement != "latest"
        && requirement != latest
        && requirement.chars().any(|char| char.is_ascii_alphabetic())
        && requirement
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || matches!(char, '-' | '_' | '.'))
}
