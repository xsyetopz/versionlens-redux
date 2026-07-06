use crate::model::SuggestionStatus::{
    Directory as StatusDirectory, DirectoryNotFound as StatusDirectoryNotFound,
    Error as StatusError, Fixed as StatusFixed, Invalid as StatusInvalid, NoMatch as StatusNoMatch,
    NotSupported as StatusNotSupported,
};
use versionlens_parsers::Dependency;

use crate::model::Suggestion;

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
mod tests;
