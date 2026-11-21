use regex_automata::meta::Regex;

use super::*;
use crate::extract::Rule::{Lines, Spec};
use crate::registry;

#[test]
fn matches_trailing_segments_case_insensitively() {
    let id = |path: &str| file_type(path).map(|f| f.id);
    assert_eq!(id("/a/requirements.txt"), Some("requirements"));
    assert_eq!(id("/a/requirements/dev.txt"), Some("requirements"));
    assert_eq!(id("C:\\a\\Requirements-Dev.TXT"), Some("requirements"));
    assert_eq!(id("/a/libraries.hxml"), Some("hxml"));
    assert_eq!(id("/a/.github/workflows/ci.yml"), Some("github-actions"));
    assert_eq!(id("/a/workflows/ci.yml"), None);
    assert_eq!(id("/a/Packages/manifest.json"), Some("unity"));
    assert_eq!(id("/a/notes.txt"), None);
}

#[test]
fn every_row_names_a_known_registry_and_valid_regexes() {
    for file in FILE_TYPES {
        assert!(
            registry::get(file.registry).is_some(),
            "{} names {}",
            file.id,
            file.registry
        );
        for rule in file.rules {
            if let Spec(_, pattern) | Lines(pattern) = rule {
                assert!(Regex::new(pattern).is_ok(), "{}: {pattern}", file.id);
            }
        }
    }
    assert!(globs_json().contains("*.hxml"));
}
