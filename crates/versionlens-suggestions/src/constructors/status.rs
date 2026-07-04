use versionlens_parsers::Dependency;

use crate::model::{Suggestion, SuggestionStatus};

pub fn no_match(dependency: Dependency) -> Suggestion {
    no_match_with_message(dependency, None)
}

pub fn no_match_with_message(dependency: Dependency, message: Option<String>) -> Suggestion {
    Suggestion {
        dependency,
        latest: message,
        resolved: None,
        status: SuggestionStatus::NoMatch,
        builds: Vec::new(),
        choices: Vec::new(),
    }
}

pub fn not_supported(dependency: Dependency) -> Suggestion {
    Suggestion {
        dependency,
        latest: None,
        resolved: None,
        status: SuggestionStatus::NotSupported,
        builds: Vec::new(),
        choices: Vec::new(),
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
        status: SuggestionStatus::Directory,
        builds: Vec::new(),
        choices: Vec::new(),
    }
}

pub fn directory_not_found(dependency: Dependency, path: String) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(path),
        resolved: None,
        status: SuggestionStatus::DirectoryNotFound,
        builds: Vec::new(),
        choices: Vec::new(),
    }
}

pub fn fixed(dependency: Dependency, value: String) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(value),
        resolved: None,
        status: SuggestionStatus::Fixed,
        builds: Vec::new(),
        choices: Vec::new(),
    }
}

pub fn error(dependency: Dependency, message: String) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(message),
        resolved: None,
        status: SuggestionStatus::Error,
        builds: Vec::new(),
        choices: Vec::new(),
    }
}

pub fn invalid(dependency: Dependency, message: String) -> Suggestion {
    Suggestion {
        dependency,
        latest: Some(message),
        resolved: None,
        status: SuggestionStatus::Invalid,
        builds: Vec::new(),
        choices: Vec::new(),
    }
}

#[cfg(test)]
mod tests;
