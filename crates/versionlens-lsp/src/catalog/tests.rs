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

#[test]
fn matches_restored_and_added_globs() {
    let id = |path: &str| file_type(path).map(|f| f.id);
    assert_eq!(id("a/dockerfile"), Some("dockerfile"));
    assert_eq!(id("a/api.dockerfile"), Some("dockerfile"));
    assert_eq!(id("compose.dev.yaml"), Some("compose"));
    assert_eq!(id(".github/workflows/a/b.yml"), Some("github-actions"));
    assert_eq!(id(".github/actions/x/action.yml"), Some("github-actions"));
    assert_eq!(
        id("a/.github/actions/x/y/action.yaml"),
        Some("github-actions")
    );
    assert_eq!(id("rust-toolchain"), Some("rust"));
    assert_eq!(id("a/.cargo/config.toml"), Some("cargo-config"));
    assert_eq!(id("flake.nix"), Some("flake"));
    assert_eq!(id("x/foo-1.0-1.rockspec"), Some("rockspec"));
    assert_eq!(id("xmake.lua"), Some("xmake"));
    assert_eq!(id("subprojects/a.wrap"), Some("meson-wrap"));
    assert_eq!(id("a.wrap"), None);
    assert_eq!(id(".mise/config.toml"), Some("mise"));
    assert_eq!(id(".tool-versions"), Some("tool-versions"));
    assert_eq!(id(".python-version"), Some("python"));
}
