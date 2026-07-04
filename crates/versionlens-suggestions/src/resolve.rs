use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_versions::{
    is_build_update, is_dotnet_requirement_parseable, is_update_available, normalized_version,
    requirement_has_empty_comparator_intersection, requirement_is_parseable,
    requirement_satisfies_latest,
};

use crate::constructors::no_match;
use crate::model::{Suggestion, SuggestionStatus};

mod github;
mod npm;

use github::github_commit_status_for_dependency;
use npm::is_npm_dist_tag_dependency;

pub fn resolve_dependency(dependency: Dependency, latest: Option<String>) -> Suggestion {
    let Some(latest) = latest else {
        return Suggestion {
            dependency,
            latest: None,
            resolved: None,
            status: SuggestionStatus::Unresolved,
            builds: Vec::new(),
            choices: Vec::new(),
        };
    };

    if dependency.ecosystem == Ecosystem::Dotnet
        && !is_dotnet_requirement_parseable(&dependency.requirement)
    {
        return no_match(dependency);
    }

    let status = resolved_status(&dependency, &latest);

    Suggestion {
        dependency,
        latest: Some(latest),
        resolved: None,
        status,
        builds: Vec::new(),
        choices: Vec::new(),
    }
}

fn resolved_status(dependency: &Dependency, latest: &str) -> SuggestionStatus {
    let requirement = comparable_requirement(dependency);

    if let Some(status) = github_commit_status_for_dependency(dependency, latest) {
        return status;
    }

    if docker_tag_update_available(dependency, latest) {
        return SuggestionStatus::UpdateAvailable;
    }

    if is_build_update(latest, requirement) {
        return SuggestionStatus::BuildAvailable;
    }

    if requirement_has_empty_comparator_intersection(requirement) {
        return SuggestionStatus::InvalidRange;
    }

    if semver_registry_requirement_is_not_parseable(dependency, requirement, latest) {
        return SuggestionStatus::NoMatch;
    }

    if is_npm_dist_tag_dependency(dependency, latest) || is_update_available(latest, requirement) {
        return SuggestionStatus::UpdateAvailable;
    }

    if requirement_satisfies_latest(requirement, latest) {
        if range_minimum_matches_latest(requirement, latest) {
            return SuggestionStatus::Current;
        }
        return SuggestionStatus::SatisfiesLatest;
    }

    SuggestionStatus::Current
}

fn comparable_requirement<'a>(dependency: &'a Dependency) -> &'a str {
    if matches!(dependency.ecosystem, Ecosystem::Deno | Ecosystem::Npm)
        && let Some(requirement) = registry_alias_requirement(&dependency.requirement)
    {
        return requirement;
    }

    &dependency.requirement
}

fn registry_alias_requirement(requirement: &str) -> Option<&str> {
    let spec = requirement
        .strip_prefix("jsr:")
        .or_else(|| requirement.strip_prefix("npm:"))?;
    let Some(split) = spec.rfind('@').filter(|index| *index > 0) else {
        return Some("");
    };
    Some(&spec[split + 1..])
}

fn semver_registry_requirement_is_not_parseable(
    dependency: &Dependency,
    requirement: &str,
    latest: &str,
) -> bool {
    !matches!(dependency.ecosystem, Ecosystem::Docker | Ecosystem::Npm)
        && dependency.hosted_url.is_none()
        && normalized_version(latest).is_some()
        && !requirement_is_parseable(requirement, latest)
}

fn range_minimum_matches_latest(requirement: &str, latest: &str) -> bool {
    let Some(minimum) = range_minimum(requirement) else {
        return false;
    };

    let latest = latest.trim_start_matches(['v', 'V']);
    latest.starts_with(&minimum)
}

fn range_minimum(requirement: &str) -> Option<String> {
    let token = range_minimum_token(requirement)?;
    let (core, suffix) = token
        .split_once('-')
        .map_or((token, None), |(core, suffix)| (core, Some(suffix)));
    let parts = core.split('.').collect::<Vec<_>>();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }

    let mut normalized = Vec::new();
    for part in parts {
        normalized.push(normalize_range_part(part)?);
    }
    while normalized.len() < 3 {
        normalized.push("0".to_owned());
    }

    let core = normalized.join(".");
    Some(match suffix {
        Some(suffix) => format!("{core}-{suffix}"),
        None => core,
    })
}

fn range_minimum_token(requirement: &str) -> Option<&str> {
    let first_range = requirement
        .trim()
        .split("||")
        .next()?
        .split(',')
        .next()?
        .trim();
    let token = first_range
        .trim_start_matches(['^', '~', '>', '<', '=', 'v', 'V'])
        .split_whitespace()
        .next()?;
    (!token.is_empty()).then_some(token)
}

fn normalize_range_part(part: &str) -> Option<String> {
    if part == "*" || part.eq_ignore_ascii_case("x") {
        return Some("0".to_owned());
    }
    part.chars()
        .all(|char| char.is_ascii_digit())
        .then(|| part.to_owned())
}

fn docker_tag_update_available(dependency: &Dependency, latest: &str) -> bool {
    if dependency.ecosystem != Ecosystem::Docker {
        return false;
    }

    let Some(latest) = docker_tag_numbers(latest) else {
        return false;
    };
    let Some(current) = docker_tag_numbers(&dependency.requirement) else {
        return false;
    };

    docker_numbers_are_newer(&latest, &current)
}

fn docker_tag_numbers(tag: &str) -> Option<Vec<u64>> {
    let version = tag.split_once('-').map_or(tag, |(version, _)| version);
    let numbers = version
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (!numbers.is_empty()).then_some(numbers)
}

fn docker_numbers_are_newer(left: &[u64], right: &[u64]) -> bool {
    let len = left.len().max(right.len());
    for index in 0..len {
        let ordering = left
            .get(index)
            .unwrap_or(&0)
            .cmp(right.get(index).unwrap_or(&0));
        if !ordering.is_eq() {
            return ordering.is_gt();
        }
    }
    false
}

#[cfg(test)]
mod tests;
