use versionlens_parsers::{Dependency, Ecosystem};

pub(crate) fn requirement_mentions_prerelease(requirement: &str) -> bool {
    requirement
        .split([' ', ',', '<', '>', '=', '^', '~', '|', '&', '(', ')'])
        .any(|part| part.contains('-') && part.chars().any(|char| char.is_ascii_digit()))
}

pub(crate) fn npm_requirement_may_be_dist_tag(dependency: &Dependency) -> bool {
    let requirement = dependency.requirement.trim();
    dependency.ecosystem == Ecosystem::Npm
        && !requirement.is_empty()
        && requirement.chars().any(|char| char.is_ascii_alphabetic())
        && requirement
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || matches!(char, '-' | '_' | '.'))
}

#[cfg(test)]
mod tests;
