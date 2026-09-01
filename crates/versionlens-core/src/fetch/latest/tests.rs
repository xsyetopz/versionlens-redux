use versionlens_model::DocumentInput;
use versionlens_model::{Dependency, Ecosystem};
use versionlens_model::{Position, Range};

use super::response_update_choices;
use crate::{ProviderSettings, RegistryUrlConfig};
use versionlens_model::Ecosystem::*;

#[test]
fn invalid_registry_url_creates_contextual_error_suggestion() {
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

    let output = session.resolve_document(DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("invalid-registry-url-creates-contextual-error-suggestion.json"),
        None,
    ));

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
