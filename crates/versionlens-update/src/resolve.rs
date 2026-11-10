use crate::suggestion::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Current as StatusCurrent,
    InvalidRange as StatusInvalidRange, NoMatch as StatusNoMatch,
    SatisfiesLatest as StatusSatisfiesLatest, Unresolved as StatusUnresolved,
    UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_model::Ecosystem::{Deno, Docker, Dotnet, Nix, Npm, Python};
use versionlens_model::{
    Dependency, VersionDialect, is_build_update, is_dotnet_requirement_parseable,
    is_npm_dist_tag_requirement, is_update_available_for_dialect, normalized_version_for_dialect,
    requirement_has_empty_comparator_intersection, requirement_is_parseable_for_dialect,
    requirement_satisfies_latest_for_dialect,
};

use crate::constructors::no_match;
use crate::suggestion::{Suggestion, SuggestionStatus};

mod github;

use github::github_commit_status_for_dependency;

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

    if matches!(dialect, VersionDialect::Semver | VersionDialect::Cargo)
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
        // The requirement allows latest. It is current unless its floor can rise.
        if crate::floor_reaches(requirement, latest, dialect) {
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

/// The version dialect that reads the requirement of `dependency`.
pub fn version_dialect(dependency: &Dependency) -> VersionDialect {
    if dependency.ecosystem == Python {
        VersionDialect::Pep440
    } else if dependency.is_cargo_dependency_requirement() {
        VersionDialect::Cargo
    } else {
        VersionDialect::Semver
    }
}

fn docker_tag_update_available(dependency: &Dependency, latest: &str) -> bool {
    if dependency.ecosystem != Docker {
        return false;
    }

    let Some(latest) = versionlens_model::numeric_segments(latest) else {
        return false;
    };
    let Some(current) = versionlens_model::numeric_segments(&dependency.requirement) else {
        return false;
    };

    docker_numbers_are_newer(&latest, &current)
}

fn docker_numbers_are_newer(left: &[u64], right: &[u64]) -> bool {
    versionlens_model::compare_numeric_segments(left, right).is_gt()
}

fn is_npm_dist_tag_dependency(dependency: &Dependency, latest: &str) -> bool {
    let requirement = dependency.requirement.trim();
    dependency.ecosystem == Npm
        && requirement != "latest"
        && requirement != latest
        && is_npm_dist_tag_requirement(requirement)
}

#[cfg(test)]
mod tests {
use crate::suggestion::SuggestionStatus::{
    Current as StatusCurrent, InvalidRange as StatusInvalidRange, NoMatch as StatusNoMatch,
    SatisfiesLatest as StatusSatisfiesLatest, UpdateAvailable as StatusUpdateAvailable,
};

use super::resolve_dependency;
use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::*;

#[test]
fn npm_dist_tag_requirement_resolves_as_update() {
    let mut current = crate::support::tests::test_dependency("typescript", "next");
    current.ecosystem = Npm;

    assert_eq!(
        resolve_dependency(current, Some("7.0.0-beta.1".to_owned())).status,
        StatusUpdateAvailable
    );
}

#[test]
fn npm_latest_dist_tag_resolves_as_current() {
    let mut current = crate::support::tests::test_dependency("typescript", "latest");
    current.ecosystem = Npm;

    assert_eq!(
        resolve_dependency(current, Some("6.0.3".to_owned())).status,
        StatusCurrent
    );
}

#[test]
fn npm_alias_specifiers_compare_embedded_version_ranges() {
    let mut current = crate::support::tests::test_dependency("chalk", "npm:chalk@^5.3.0");
    current.ecosystem = Npm;

    assert_eq!(
        resolve_dependency(current, Some("6.0.0".to_owned())).status,
        StatusUpdateAvailable
    );
}

#[test]
fn invalid_dotnet_requirements_resolve_as_no_match() {
    let mut current = crate::support::tests::test_dependency("Test.Package", "invalid");
    current.ecosystem = Dotnet;

    let suggestion = resolve_dependency(current, Some("1.1.0".to_owned()));

    assert_eq!(suggestion.status, StatusNoMatch);
}

#[test]
fn invalid_semver_requirements_resolve_as_no_match() {
    let current = crate::support::tests::test_dependency("serde", "not-a-version");

    let suggestion = resolve_dependency(current, Some("1.1.0".to_owned()));

    assert_eq!(suggestion.status, StatusNoMatch);
}

#[test]
fn empty_semver_ranges_resolve_as_invalid_range() {
    for requirement in [">1 <1", ">1.0.0 <1.0.1", ">2 <1"] {
        let current = crate::support::tests::test_dependency("typescript", requirement);

        assert_eq!(
            resolve_dependency(current, Some("5.0.0".to_owned())).status,
            StatusInvalidRange
        );
    }
}

#[test]
fn ranges_whose_minimum_matches_latest_resolve_as_current_like_upstream() {
    let current = crate::support::tests::test_dependency("typescript", "~1.1.0");

    assert_eq!(
        resolve_dependency(current, Some("1.1.0".to_owned())).status,
        StatusCurrent
    );
}

#[test]
fn range_minimum_is_compared_as_a_version_not_as_a_prefix() {
    let current = crate::support::tests::test_dependency("typescript", "^1.2.1");

    assert_eq!(
        resolve_dependency(current, Some("1.2.10".to_owned())).status,
        StatusSatisfiesLatest
    );
}

fn assert_github_commit_status(
    ecosystem: Ecosystem,
    name: &str,
    current_requirement: &str,
    stale_requirement: &str,
) {
    let mut current = crate::support::tests::test_dependency(name, current_requirement);
    current.ecosystem = ecosystem;
    current.hosted_url = Some(format!("https://api.github.com/repos/{name}/commits"));

    let mut stale = current.clone();
    stale.requirement = stale_requirement.to_owned();

    assert_eq!(
        resolve_dependency(current, Some("abcdef1".to_owned())).status,
        StatusCurrent
    );
    assert_eq!(
        resolve_dependency(stale, Some("abcdef1".to_owned())).status,
        StatusUpdateAvailable
    );
}

#[test]
fn ruby_github_commit_refs_compare_as_shas() {
    assert_github_commit_status(Ruby, "rspec/rspec-core", "abcdef1234567890", "1234567");
}

#[test]
fn npm_github_commit_refs_compare_as_shas() {
    assert_github_commit_status(Npm, "owner/commit", "abcdef1234567890", "1234567");
}

#[test]
fn python_pep440_constraints_and_releases_are_resolved() {
    let mut excluded = crate::support::tests::test_dependency("demo", ">=1!2.0.0.0,!=1!2.1.0.0");
    excluded.ecosystem = Python;
    assert_eq!(
        resolve_dependency(excluded, Some("1!2.0.0.4+linux.1".to_owned())).status,
        crate::suggestion::SuggestionStatus::SatisfiesLatest,
    );

    let mut prerelease = crate::support::tests::test_dependency("demo", ">=1.0rc1,<2.0");
    prerelease.ecosystem = Python;
    assert_eq!(
        resolve_dependency(prerelease, Some("1.1-rc.2".to_owned())).status,
        crate::suggestion::SuggestionStatus::SatisfiesLatest,
    );
}

#[test]
fn python_provider_versions_older_than_declared_bounds_are_not_updates() {
    for requirement in ["2.0", "==2.0", ">=2.0,<3.0"] {
        let mut current = crate::support::tests::test_dependency("demo", requirement);
        current.ecosystem = Python;
        assert_eq!(
            resolve_dependency(current, Some("1.9.post1".to_owned())).status,
            StatusCurrent,
        );
    }
}
}
