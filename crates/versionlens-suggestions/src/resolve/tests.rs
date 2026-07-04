use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_vscode_model::{Position, Range};

use crate::model::SuggestionStatus;

use super::resolve_dependency;

#[test]
fn npm_dist_tag_requirement_resolves_as_update() {
    let mut current = dependency("typescript", "next");
    current.ecosystem = Ecosystem::Npm;

    assert_eq!(
        resolve_dependency(current, Some("7.0.0-beta.1".to_owned())).status,
        SuggestionStatus::UpdateAvailable
    );
}

#[test]
fn npm_latest_dist_tag_resolves_as_current() {
    let mut current = dependency("typescript", "latest");
    current.ecosystem = Ecosystem::Npm;

    assert_eq!(
        resolve_dependency(current, Some("6.0.3".to_owned())).status,
        SuggestionStatus::Current
    );
}

#[test]
fn npm_alias_specifiers_compare_embedded_version_ranges() {
    let mut current = dependency("chalk", "npm:chalk@^5.3.0");
    current.ecosystem = Ecosystem::Npm;

    assert_eq!(
        resolve_dependency(current, Some("6.0.0".to_owned())).status,
        SuggestionStatus::UpdateAvailable
    );
}

#[test]
fn invalid_dotnet_requirements_resolve_as_no_match() {
    let mut current = dependency("Test.Package", "invalid");
    current.ecosystem = Ecosystem::Dotnet;

    let suggestion = resolve_dependency(current, Some("1.1.0".to_owned()));

    assert_eq!(suggestion.status, SuggestionStatus::NoMatch);
}

#[test]
fn invalid_semver_requirements_resolve_as_no_match() {
    let current = dependency("serde", "not-a-version");

    let suggestion = resolve_dependency(current, Some("1.1.0".to_owned()));

    assert_eq!(suggestion.status, SuggestionStatus::NoMatch);
}

#[test]
fn empty_semver_ranges_resolve_as_invalid_range() {
    for requirement in [">1 <1", ">1.0.0 <1.0.1", ">2 <1"] {
        let current = dependency("typescript", requirement);

        assert_eq!(
            resolve_dependency(current, Some("5.0.0".to_owned())).status,
            SuggestionStatus::InvalidRange
        );
    }
}

#[test]
fn ranges_whose_minimum_matches_latest_resolve_as_current_like_upstream() {
    let current = dependency("typescript", "~1.1.0");

    assert_eq!(
        resolve_dependency(current, Some("1.1.0".to_owned())).status,
        SuggestionStatus::Current
    );
}

#[test]
fn ruby_github_commit_refs_compare_as_shas() {
    let mut current = dependency("rspec/rspec-core", "abcdef1234567890");
    current.ecosystem = Ecosystem::Ruby;
    current.hosted_url = Some("https://api.github.com/repos/rspec/rspec-core/commits".to_owned());

    let mut stale = current.clone();
    stale.requirement = "main".to_owned();

    assert_eq!(
        resolve_dependency(current, Some("abcdef1".to_owned())).status,
        SuggestionStatus::Current
    );
    assert_eq!(
        resolve_dependency(stale, Some("abcdef1".to_owned())).status,
        SuggestionStatus::UpdateAvailable
    );
}

#[test]
fn npm_github_commit_refs_compare_as_shas() {
    let mut current = dependency("owner/commit", "abcdef1234567890");
    current.ecosystem = Ecosystem::Npm;
    current.hosted_url = Some("https://api.github.com/repos/owner/commit/commits".to_owned());

    let mut stale = current.clone();
    stale.requirement = "1234567".to_owned();

    assert_eq!(
        resolve_dependency(current, Some("abcdef1".to_owned())).status,
        SuggestionStatus::Current
    );
    assert_eq!(
        resolve_dependency(stale, Some("abcdef1".to_owned())).status,
        SuggestionStatus::UpdateAvailable
    );
}

fn dependency(name: &str, requirement: &str) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Ecosystem::Cargo,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: empty_range(),
        requirement_range: empty_range(),
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
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
