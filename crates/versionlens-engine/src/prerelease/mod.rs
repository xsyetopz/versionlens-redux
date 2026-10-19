use versionlens_model::Ecosystem::{Composer, Npm};
use versionlens_model::{Dependency, is_npm_dist_tag_requirement};

pub(crate) fn requirement_mentions_prerelease(requirement: &str) -> bool {
    requirement
        .split([' ', ',', '<', '>', '=', '^', '~', '|', '&', '(', ')'])
        .any(|part| part.contains('-') && part.chars().any(|char| char.is_ascii_digit()))
}

pub(crate) fn dependency_allows_prereleases(dependency: &Dependency) -> bool {
    requirement_mentions_prerelease(&dependency.requirement)
        || (dependency.ecosystem == Composer
            && composer_stability_flag_allows_prereleases(&dependency.requirement_suffix))
}

fn composer_stability_flag_allows_prereleases(suffix: &str) -> bool {
    suffix.split([' ', '#']).any(|part| {
        let flag = part.trim().trim_start_matches('@');
        matches!(
            flag.to_ascii_lowercase().as_str(),
            "dev" | "alpha" | "beta" | "rc"
        )
    })
}

pub(crate) fn npm_requirement_may_be_dist_tag(dependency: &Dependency) -> bool {
    dependency.ecosystem == Npm && is_npm_dist_tag_requirement(&dependency.requirement)
}

#[cfg(test)]
mod tests;
