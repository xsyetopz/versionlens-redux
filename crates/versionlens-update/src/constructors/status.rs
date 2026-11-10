use crate::suggestion::SuggestionStatus::{
    Directory as StatusDirectory, DirectoryNotFound as StatusDirectoryNotFound,
    Error as StatusError, Fixed as StatusFixed, Invalid as StatusInvalid, NoMatch as StatusNoMatch,
    NotSupported as StatusNotSupported, RateLimited as StatusRateLimited,
};
use versionlens_model::Dependency;

use crate::suggestion::Suggestion;

pub fn no_match(dependency: Dependency) -> Suggestion {
    no_match_with_message(dependency, None)
}

pub fn no_match_with_message(dependency: Dependency, message: Option<String>) -> Suggestion {
    Suggestion {
        dependency,
        latest: message,
        resolved: None,
        status: StatusNoMatch,
        builds: vec![],
        choices: vec![],
    }
}

pub fn not_supported(dependency: Dependency) -> Suggestion {
    Suggestion {
        dependency,
        latest: None,
        resolved: None,
        status: StatusNotSupported,
        builds: vec![],
        choices: vec![],
    }
}

pub fn directory(
    dependency: Dependency,
    display_path: String,
    resolved_path: String,
) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(display_path),
        resolved: Some(resolved_path),
        status: StatusDirectory,
        builds: vec![],
        choices: vec![],
    }
}

pub fn directory_not_found(dependency: Dependency, path: String) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(path),
        resolved: None,
        status: StatusDirectoryNotFound,
        builds: vec![],
        choices: vec![],
    }
}

pub fn fixed(dependency: Dependency, value: String) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(value),
        resolved: None,
        status: StatusFixed,
        builds: vec![],
        choices: vec![],
    }
}

pub fn error(dependency: Dependency, message: String) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(message),
        resolved: None,
        status: StatusError,
        builds: vec![],
        choices: vec![],
    }
}

/// A dependency the registry would not answer for until its rate limit
/// resets. `message` is the reset time, shown where an error would be.
pub fn rate_limited(dependency: Dependency, message: String) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(message),
        resolved: None,
        status: StatusRateLimited,
        builds: vec![],
        choices: vec![],
    }
}

pub fn invalid(dependency: Dependency, message: String) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(message),
        resolved: None,
        status: StatusInvalid,
        builds: vec![],
        choices: vec![],
    }
}

#[cfg(test)]
mod tests {
use crate::suggestion::SuggestionStatus::{
    Directory as StatusDirectory, DirectoryNotFound as StatusDirectoryNotFound,
    Error as StatusError, Fixed as StatusFixed, NoMatch as StatusNoMatch,
};

use super::{directory, directory_not_found, error, fixed, no_match, no_match_with_message};

#[test]
fn no_match_marks_dependency_no_match() {
    let suggestion = no_match(crate::support::tests::test_dependency("serde", "1.0.0"));

    assert_eq!(suggestion.status, StatusNoMatch);
    assert_eq!(suggestion.latest, None);
}

#[test]
fn no_match_can_carry_a_message() {
    let suggestion = no_match_with_message(
        crate::support::tests::test_dependency("serde", "1.0.0"),
        Some("not supported".to_owned()),
    );

    assert_eq!(suggestion.status, StatusNoMatch);
    assert_eq!(suggestion.latest.as_deref(), Some("not supported"));
}

#[test]
fn directory_marks_dependency_directory() {
    let suggestion = directory(
        crate::support::tests::test_dependency("local", "file:../local"),
        "../local".to_owned(),
        "/repo/local".to_owned(),
    );

    assert_eq!(suggestion.status, StatusDirectory);
    assert_eq!(suggestion.latest.as_deref(), Some("../local"));
    assert_eq!(suggestion.resolved.as_deref(), Some("/repo/local"));
}

#[test]
fn directory_not_found_marks_dependency_directory_not_found() {
    let suggestion = directory_not_found(
        crate::support::tests::test_dependency("local", "file:../missing"),
        "../missing".to_owned(),
    );

    assert_eq!(suggestion.status, StatusDirectoryNotFound);
    assert_eq!(suggestion.latest.as_deref(), Some("../missing"));
    assert_eq!(suggestion.resolved, None);
}

#[test]
fn fixed_marks_dependency_fixed() {
    let suggestion = fixed(
        crate::support::tests::test_dependency("remote", "git repository"),
        "git repository".to_owned(),
    );

    assert_eq!(suggestion.status, StatusFixed);
    assert_eq!(suggestion.latest.as_deref(), Some("git repository"));
}

#[test]
fn error_marks_dependency_error() {
    let suggestion = error(
        crate::support::tests::test_dependency("serde", "1.0.0"),
        "not found".to_owned(),
    );

    assert_eq!(suggestion.status, StatusError);
    assert_eq!(suggestion.latest.as_deref(), Some("not found"));
}
}
