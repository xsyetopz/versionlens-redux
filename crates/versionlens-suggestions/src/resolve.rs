use crate::suggestion::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Current as StatusCurrent,
    InvalidRange as StatusInvalidRange, NoMatch as StatusNoMatch,
    SatisfiesLatest as StatusSatisfiesLatest, Unresolved as StatusUnresolved,
    UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::{Deno, Docker, Dotnet, Nix, Npm, Python};
use versionlens_versions::{
    VersionDialect, is_build_update, is_dotnet_requirement_parseable,
    is_update_available_for_dialect, normalized_version_for_dialect,
    requirement_has_empty_comparator_intersection, requirement_is_parseable_for_dialect,
    requirement_satisfies_latest_for_dialect,
};

use crate::constructors::no_match;
use crate::suggestion::{Suggestion, SuggestionStatus};
use crate::support;

mod github;
mod npm;

use github::github_commit_status_for_dependency;
use npm::is_npm_dist_tag_dependency;

type NumericSegments = Vec<u64>;

pub fn resolve_dependency(dependency: Dependency, latest: Option<String>) -> Suggestion {
    let Some(latest) = latest else {
        return Suggestion {
            dependency,
            latest: None,
            resolved: None,
            status: StatusUnresolved,
            builds: vec![],
            choices: vec![],
        };
    };

    if dependency.ecosystem == Dotnet && !is_dotnet_requirement_parseable(&dependency.requirement) {
        return no_match(dependency);
    }

    let status = resolved_status(&dependency, &latest);

    Suggestion {
        dependency,
        latest: Some(latest),
        resolved: None,
        status,
        builds: vec![],
        choices: vec![],
    }
}

fn resolved_status(dependency: &Dependency, latest: &str) -> SuggestionStatus {
    let requirement = comparable_requirement(dependency);
    let dialect = version_dialect(dependency);

    if let Some(status) = github_commit_status_for_dependency(dependency, latest) {
        return status;
    }

    if docker_tag_update_available(dependency, latest) {
        return StatusUpdateAvailable;
    }

    if nix_release_update_available(dependency, latest, requirement) {
        return StatusUpdateAvailable;
    }

    if is_build_update(latest, requirement) {
        return StatusBuildAvailable;
    }

    if dialect == VersionDialect::Semver
        && requirement_has_empty_comparator_intersection(requirement)
    {
        return StatusInvalidRange;
    }

    if registry_requirement_is_not_parseable(dependency, requirement, latest, dialect) {
        return StatusNoMatch;
    }

    if is_npm_dist_tag_dependency(dependency, latest)
        || is_update_available_for_dialect(latest, requirement, dialect)
    {
        return StatusUpdateAvailable;
    }

    if requirement_satisfies_latest_for_dialect(requirement, latest, dialect) {
        if range_minimum_matches_latest(requirement, latest) {
            return StatusCurrent;
        }
        return StatusSatisfiesLatest;
    }

    StatusCurrent
}

fn nix_release_update_available(dependency: &Dependency, latest: &str, requirement: &str) -> bool {
    if dependency.ecosystem != Nix {
        return false;
    }
    let Some(latest) = comparable_nix_release(latest) else {
        return false;
    };
    let Some(current) = comparable_nix_release(requirement) else {
        return false;
    };
    latest > current
}

fn comparable_nix_release(value: &str) -> Option<NumericSegments> {
    let value = value
        .strip_prefix("nixos-")
        .or_else(|| value.strip_prefix("release-"))
        .unwrap_or(value);
    let core = value.split_once('-').map_or(value, |(core, _)| core);
    let parts = core
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    (!parts.is_empty()).then_some(parts)
}

fn comparable_requirement<'a>(dependency: &'a Dependency) -> &'a str {
    if matches!(dependency.ecosystem, Deno | Npm)
        && let Some(requirement) =
            versionlens_model::registry_alias_requirement(&dependency.requirement)
    {
        return requirement;
    }

    &dependency.requirement
}

fn registry_requirement_is_not_parseable(
    dependency: &Dependency,
    requirement: &str,
    latest: &str,
    dialect: VersionDialect,
) -> bool {
    !matches!(dependency.ecosystem, Docker | Npm)
        && dependency.hosted_url.is_none()
        && normalized_version_for_dialect(latest, dialect).is_some()
        && !requirement_is_parseable_for_dialect(requirement, latest, dialect)
}

fn version_dialect(dependency: &Dependency) -> VersionDialect {
    if dependency.ecosystem == Python {
        VersionDialect::Pep440
    } else {
        VersionDialect::Semver
    }
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
    support::normalize_version_token(token)
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

fn docker_tag_update_available(dependency: &Dependency, latest: &str) -> bool {
    if dependency.ecosystem != Docker {
        return false;
    }

    let Some(latest) = versionlens_versions::numeric_segments(latest) else {
        return false;
    };
    let Some(current) = versionlens_versions::numeric_segments(&dependency.requirement) else {
        return false;
    };

    docker_numbers_are_newer(&latest, &current)
}

fn docker_numbers_are_newer(left: &[u64], right: &[u64]) -> bool {
    versionlens_versions::compare_numeric_segments(left, right).is_gt()
}

#[cfg(test)]
mod tests;
