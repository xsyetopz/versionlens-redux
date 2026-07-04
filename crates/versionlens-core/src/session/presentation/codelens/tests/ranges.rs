use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem};

use crate::{ProviderSettings, RegistryResponseInput, SessionConfig, VersionLensSession};

use super::test_indicators;

#[test]
fn code_lenses_offer_minor_update_choices_for_tilde_ranges() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"~1.1"}}"#.to_owned(),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{
              "dist-tags": { "latest": "2.2.2" },
              "versions": {
                "1.1.0": {},
                "1.1.1": {},
                "1.1.2": {},
                "1.2.0": {},
                "1.2.2": {},
                "2.0.0": {},
                "2.2.2": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = output
        .code_lenses
        .iter()
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        titles,
        [
            "M satisfies 1.1.2",
            "U latest 2.2.2",
            "U minor 1.2.2",
            "U bump 1.1.2"
        ]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "2.2.2"],
            vec!["updateMinor", "1.2.2"],
            vec!["update", "1.1.2"]
        ]
    );
}
