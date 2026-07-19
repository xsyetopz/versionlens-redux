use crate::suggestion::SuggestionStatus::{
    Current as StatusCurrent, InvalidRange as StatusInvalidRange, NoMatch as StatusNoMatch,
    UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_model::Dependency;
use versionlens_model::{Position, Range};

use super::resolve_dependency;
use versionlens_model::Ecosystem::{Cargo, Dotnet, Npm, Python, Ruby};

#[test]
fn npm_dist_tag_requirement_resolves_as_update() {
    let mut current = dependency("typescript", "next");
    current.ecosystem = Npm;

    assert_eq!(
        resolve_dependency(current, Some("7.0.0-beta.1".to_owned())).status,
        StatusUpdateAvailable
    );
}

#[test]
fn npm_latest_dist_tag_resolves_as_current() {
    let mut current = dependency("typescript", "latest");
    current.ecosystem = Npm;

    assert_eq!(
        resolve_dependency(current, Some("6.0.3".to_owned())).status,
        StatusCurrent
    );
}

#[test]
fn npm_alias_specifiers_compare_embedded_version_ranges() {
    let mut current = dependency("chalk", "npm:chalk@^5.3.0");
    current.ecosystem = Npm;

    assert_eq!(
        resolve_dependency(current, Some("6.0.0".to_owned())).status,
        StatusUpdateAvailable
    );
}

#[test]
fn invalid_dotnet_requirements_resolve_as_no_match() {
    let mut current = dependency("Test.Package", "invalid");
    current.ecosystem = Dotnet;

    let suggestion = resolve_dependency(current, Some("1.1.0".to_owned()));

    assert_eq!(suggestion.status, StatusNoMatch);
}

#[test]
fn invalid_semver_requirements_resolve_as_no_match() {
    let current = dependency("serde", "not-a-version");

    let suggestion = resolve_dependency(current, Some("1.1.0".to_owned()));

    assert_eq!(suggestion.status, StatusNoMatch);
}

#[test]
fn empty_semver_ranges_resolve_as_invalid_range() {
    for requirement in [">1 <1", ">1.0.0 <1.0.1", ">2 <1"] {
        let current = dependency("typescript", requirement);

        assert_eq!(
            resolve_dependency(current, Some("5.0.0".to_owned())).status,
            StatusInvalidRange
        );
    }
}

#[test]
fn ranges_whose_minimum_matches_latest_resolve_as_current_like_upstream() {
    let current = dependency("typescript", "~1.1.0");

    assert_eq!(
        resolve_dependency(current, Some("1.1.0".to_owned())).status,
        StatusCurrent
    );
}

#[test]
fn ruby_github_commit_refs_compare_as_shas() {
    let mut current = dependency("rspec/rspec-core", "abcdef1234567890");
    current.ecosystem = Ruby;
    current.hosted_url = Some("https://api.github.com/repos/rspec/rspec-core/commits".to_owned());

    let mut stale = current.clone();
    stale.requirement = "main".to_owned();

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
fn npm_github_commit_refs_compare_as_shas() {
    let mut current = dependency("owner/commit", "abcdef1234567890");
    current.ecosystem = Npm;
    current.hosted_url = Some("https://api.github.com/repos/owner/commit/commits".to_owned());

    let mut stale = current.clone();
    stale.requirement = "1234567".to_owned();

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
fn python_pep440_constraints_and_releases_are_resolved() {
    let mut excluded = dependency("demo", ">=1!2.0.0.0,!=1!2.1.0.0");
    excluded.ecosystem = Python;
    assert_eq!(
        resolve_dependency(excluded, Some("1!2.0.0.4+linux.1".to_owned())).status,
        crate::suggestion::SuggestionStatus::SatisfiesLatest,
    );

    let mut prerelease = dependency("demo", ">=1.0rc1,<2.0");
    prerelease.ecosystem = Python;
    assert_eq!(
        resolve_dependency(prerelease, Some("1.1-rc.2".to_owned())).status,
        crate::suggestion::SuggestionStatus::SatisfiesLatest,
    );
}

#[test]
fn python_provider_versions_older_than_declared_bounds_are_not_updates() {
    for requirement in ["2.0", "==2.0", ">=2.0,<3.0"] {
        let mut current = dependency("demo", requirement);
        current.ecosystem = Python;
        assert_eq!(
            resolve_dependency(current, Some("1.9.post1".to_owned())).status,
            StatusCurrent,
        );
    }
}

fn dependency(name: &str, requirement: &str) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Cargo,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: empty_range(),
        requirement_range: empty_range(),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn empty_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}
