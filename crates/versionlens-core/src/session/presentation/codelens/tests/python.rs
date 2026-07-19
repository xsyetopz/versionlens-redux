use versionlens_model::DocumentInput;
use versionlens_model::Ecosystem::Python;

use crate::{RegistryResponseInput, SessionConfig};

use super::test_indicators;

#[test]
fn pyproject_ranges_that_admit_latest_offer_updates_from_canonical_pypi_releases() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: r#"
[project]
dependencies = ["httpx>=0.27,<1"]

[project.optional-dependencies]
test = ["httpcore>=0.27,<1"]
"#
        .to_owned(),
        workspace_root: None,
    };
    let body = r#"{
      "info": { "version": "0.28.1" },
      "releases": {
        "0.27.0": [],
        "0.28.1": [{ "yanked": false }]
      }
    }"#
    .to_owned();

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[
            RegistryResponseInput {
                package: "httpx".to_owned(),
                ecosystem: Python,
                body: body.clone(),
            },
            RegistryResponseInput {
                package: "httpcore".to_owned(),
                ecosystem: Python,
                body,
            },
        ],
    );

    assert_eq!(
        output
            .suggestions
            .iter()
            .map(|suggestion| suggestion.status.as_str())
            .collect::<Vec<_>>(),
        ["satisfiesLatest", "satisfiesLatest"]
    );

    let lenses = session.analyze_document(input);
    assert_eq!(
        lenses
            .code_lenses
            .iter()
            .map(|lens| (lens.title.as_str(), lens.command.as_str()))
            .collect::<Vec<_>>(),
        [
            ("S satisfies latest 0.28.1", ""),
            (
                "U latest 0.28.1",
                "versionlens.suggestion.onUpdateDependency"
            ),
            ("S satisfies latest 0.28.1", ""),
            (
                "U latest 0.28.1",
                "versionlens.suggestion.onUpdateDependency"
            )
        ]
    );
}

#[test]
fn requirements_ranges_offer_updates_from_canonical_pypi_releases() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: "httpx>=0.27,<1\n".to_owned(),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "httpx".to_owned(),
            ecosystem: Python,
            body: r#"{"info":{"version":"0.28.1"},"releases":{"0.27.0":[],"0.28.1":[{"yanked":false}]}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "satisfiesLatest");
    let lenses = session.analyze_document(input);
    assert_eq!(
        lenses
            .code_lenses
            .iter()
            .map(|lens| (lens.title.as_str(), lens.command.as_str()))
            .collect::<Vec<_>>(),
        [
            ("S satisfies latest 0.28.1", ""),
            (
                "U latest 0.28.1",
                "versionlens.suggestion.onUpdateDependency"
            )
        ]
    );
}
