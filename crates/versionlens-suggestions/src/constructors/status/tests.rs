use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_vscode_model::{Position, Range};

use crate::model::SuggestionStatus;

use super::{directory, directory_not_found, error, fixed, no_match, no_match_with_message};

#[test]
fn no_match_marks_dependency_no_match() {
    let suggestion = no_match(dependency("serde", "1.0.0"));

    assert_eq!(suggestion.status, SuggestionStatus::NoMatch);
    assert_eq!(suggestion.latest, None);
}

#[test]
fn no_match_can_carry_a_message() {
    let suggestion = no_match_with_message(
        dependency("serde", "1.0.0"),
        Some("not supported".to_owned()),
    );

    assert_eq!(suggestion.status, SuggestionStatus::NoMatch);
    assert_eq!(suggestion.latest.as_deref(), Some("not supported"));
}

#[test]
fn directory_marks_dependency_directory() {
    let suggestion = directory(
        dependency("local", "file:../local"),
        "../local".to_owned(),
        "/repo/local".to_owned(),
    );

    assert_eq!(suggestion.status, SuggestionStatus::Directory);
    assert_eq!(suggestion.latest.as_deref(), Some("../local"));
    assert_eq!(suggestion.resolved.as_deref(), Some("/repo/local"));
}

#[test]
fn directory_not_found_marks_dependency_directory_not_found() {
    let suggestion = directory_not_found(
        dependency("local", "file:../missing"),
        "../missing".to_owned(),
    );

    assert_eq!(suggestion.status, SuggestionStatus::DirectoryNotFound);
    assert_eq!(suggestion.latest.as_deref(), Some("../missing"));
    assert_eq!(suggestion.resolved, None);
}

#[test]
fn fixed_marks_dependency_fixed() {
    let suggestion = fixed(
        dependency("remote", "git repository"),
        "git repository".to_owned(),
    );

    assert_eq!(suggestion.status, SuggestionStatus::Fixed);
    assert_eq!(suggestion.latest.as_deref(), Some("git repository"));
}

#[test]
fn error_marks_dependency_error() {
    let suggestion = error(dependency("serde", "1.0.0"), "not found".to_owned());

    assert_eq!(suggestion.status, SuggestionStatus::Error);
    assert_eq!(suggestion.latest.as_deref(), Some("not found"));
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
