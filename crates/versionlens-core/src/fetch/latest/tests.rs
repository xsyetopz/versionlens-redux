use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::DocumentInput;
use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_vscode_model::{Position, Range};

use super::response_update_choices;
use crate::{ProviderSettings, RegistryUrlConfig, SessionConfig};
use versionlens_parsers::Ecosystem::{Composer, Npm};

#[test]
fn invalid_registry_url_creates_contextual_error_suggestion() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Npm,
                url: "not a url".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.resolve_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("invalid-registry-url-creates-contextual-error-suggestion.json"),
        workspace_root: None,
    });

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
    let dependency = Dependency {
        name: "dplyr".to_owned(),
        requirement: "1.0.0".to_owned(),
        ecosystem: Ecosystem::Cran,
        group: "Imports".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: empty_range(),
        requirement_range: empty_range(),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    };
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
    let dependency = Dependency {
        name: "acme/target".to_owned(),
        requirement: "1.0.0".to_owned(),
        ecosystem: Composer,
        group: "require".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: empty_range(),
        requirement_range: empty_range(),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    };
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
    let path = repo_root()
        .join("tests/fixtures/core/fetch/latest/tests")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
