use versionlens_model::CanonicalReference;
use versionlens_model::DocumentInput;
use versionlens_model::{Dependency, Ecosystem};
use versionlens_model::{Position, Range};

use super::github::github_action_reference_is_proven_by_exact_ref;
use super::response_update_choices;
use crate::{ProviderSettings, RegistryUrlConfig};
use versionlens_model::Ecosystem::*;

#[tokio::test]
async fn invalid_registry_url_creates_contextual_error_suggestion() {
    let session = crate::support::tests::session_with_provider_settings(
        ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Npm,
                url: "not a url".to_owned(),
            }],
            ..crate::default()
        },
        false,
    );

    let output = session
        .resolve_document(DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("invalid-registry-url-creates-contextual-error-suggestion.json"),
            None,
        ))
        .await;

    assert_eq!(output.suggestions[0].status, "error");
    assert!(
        output.suggestions[0]
            .latest
            .as_deref()
            .is_some_and(|message| message.contains("failed to fetch registry URL")),
    );
}

#[test]
fn cran_update_choices_exclude_versions_from_other_packages() {
    let dependency = update_choice_dependency("dplyr", Ecosystem::Cran, "Imports");
    let body = "Package: dplyr\nVersion: 1.0.0\n\nPackage: dplyr\nVersion: 1.1.4\n\nPackage: unrelated\nVersion: 2.0.0\n";

    let choices = response_update_choices(&dependency, "1.1.4", body, false, &[]);

    assert_eq!(
        choices
            .iter()
            .map(|choice| choice.version.as_str())
            .collect::<Vec<_>>(),
        ["1.1.4"]
    );
}

#[test]
fn composer_update_choices_exclude_versions_from_other_packages() {
    let dependency = update_choice_dependency("acme/target", Composer, "require");
    let body = r#"{
      "packages": {
        "acme/target": [{"version": "1.0.0"}, {"version": "1.1.0"}],
        "acme/unrelated": [{"version": "9.0.0"}]
      }
    }"#;

    let choices = response_update_choices(&dependency, "1.1.0", body, false, &[]);

    assert_eq!(
        choices
            .iter()
            .map(|choice| choice.version.as_str())
            .collect::<Vec<_>>(),
        ["1.1.0"]
    );
}

#[test]
fn annotated_github_tag_objects_require_commit_dereferencing() {
    let reference = CanonicalReference::GitHubActionSha {
        commit: "8216e11d8cd9b42fe925c852af8e76311ff067ac".to_owned(),
        tag: "v2".to_owned(),
        separator: " # ".to_owned(),
    };
    let body = r#"{
      "ref": "refs/tags/v2",
      "object": {
        "type": "tag",
        "sha": "8216e11d8cd9b42fe925c852af8e76311ff067ac"
      }
    }"#;

    assert!(!github_action_reference_is_proven_by_exact_ref(
        &reference, body
    ));
}

#[test]
fn exact_github_ref_must_match_both_annotation_and_sha() {
    let reference = CanonicalReference::GitHubActionSha {
        commit: "8216e11".to_owned(),
        tag: "v2".to_owned(),
        separator: " # ".to_owned(),
    };

    assert!(!github_action_reference_is_proven_by_exact_ref(
        &reference,
        r#"{"ref":"refs/tags/v3","object":{"type":"tag","sha":"8216e11d8cd9b42fe925c852af8e76311ff067ac"}}"#
    ));
    assert!(!github_action_reference_is_proven_by_exact_ref(
        &reference,
        r#"{"ref":"refs/tags/v2","object":{"type":"tag","sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}"#
    ));
    assert!(github_action_reference_is_proven_by_exact_ref(
        &reference,
        r#"{"ref":"refs/tags/v2","object":{"type":"commit","sha":"8216e11d8cd9b42fe925c852af8e76311ff067ac"}}"#
    ));
    assert!(!github_action_reference_is_proven_by_exact_ref(
        &reference,
        r#"{"ref":"refs/tags/v2","object":{"sha":"8216e11d8cd9b42fe925c852af8e76311ff067ac"}}"#
    ));
    for sha in ["8216e11", "8216e11not-a-commit-object-sha0000000000000"] {
        let body = serde_json::json!({"ref":"refs/tags/v2","object":{"type":"commit","sha":sha}})
            .to_string();
        assert!(!github_action_reference_is_proven_by_exact_ref(
            &reference, &body
        ));
    }
}

#[test]
fn action_commit_families_require_complete_commits_and_ignore_hex_case() {
    let reference = CanonicalReference::GitHubActionCommit {
        commit: "abcdef0".to_owned(),
    };
    for (sha, expected) in [
        ("abcdef0", None),
        ("abcdef000000000000000000000000000000000x", None),
        ("ABCDEF0000000000000000000000000000000000", Some("v1.1.0")),
    ] {
        let body = serde_json::json!([
            {"name":"v1.0.0","commit":{"sha":sha.to_ascii_lowercase()}},
            {"name":"v1.1.0","commit":{"sha":sha}}
        ])
        .to_string();
        assert_eq!(
            super::github::github_action_tag(&reference, &body).as_deref(),
            expected
        );
    }
}

fn update_choice_dependency(name: &str, ecosystem: Ecosystem, group: &str) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: "1.0.0".to_owned(),
        ecosystem,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: empty_range(),
        requirement_range: empty_range(),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

fn empty_range() -> Range {
    let position = Position {
        line: 0,
        character: 0,
    };
    Range {
        start: position,
        end: position,
    }
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/core-scenarios/fetch/latest/tests", name)
}
