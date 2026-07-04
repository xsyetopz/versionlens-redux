use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem};

use crate::{ProviderSettings, RegistryResponseInput, SessionConfig, VersionLensSession};

use super::test_indicators;

#[test]
fn code_lenses_label_latest_dist_tag_prerelease_for_missing_fixed_versions() {
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
        text: r#"{"dependencies":{"left-pad":"4.0.0"}}"#.to_owned(),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{
              "dist-tags": { "latest": "4.0.0-next" },
              "versions": {
                "0.0.5": {},
                "0.0.6": {},
                "1.1.0-alpha.1": {},
                "4.0.0-next": {}
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
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(titles, ["N no match", "U latest prerelease 4.0.0-next"]);
    assert_eq!(arguments, [vec!["update", "4.0.0-next"]]);
}

#[test]
fn npm_invalid_tag_name_error_offers_latest_dist_tag_update() {
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
        text: r#"{"dependencies":{"left-pad":"bad tag"}}"#.to_owned(),
        workspace_root: None,
    };

    let resolved = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"status":"EINVALIDTAGNAME"}"#.to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let commands = output
        .code_lenses
        .iter()
        .map(|lens| lens.command.as_str())
        .collect::<Vec<_>>();
    let arguments = output
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(resolved.suggestions[0].status, "invalid");
    assert_eq!(titles, ["E invalid version", "U latest latest"]);
    assert_eq!(commands, ["", "versionlens.suggestion.onUpdateDependency"]);
    assert_eq!(arguments, [vec!["update", "latest"]]);
}

#[test]
fn npm_unsupported_protocol_error_uses_not_supported_status() {
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
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: None,
    };

    let resolved = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"status":"EUNSUPPORTEDPROTOCOL"}"#.to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let commands = output
        .code_lenses
        .iter()
        .map(|lens| lens.command.as_str())
        .collect::<Vec<_>>();

    assert_eq!(resolved.suggestions[0].status, "notSupported");
    assert_eq!(titles, ["N not supported"]);
    assert_eq!(commands, [""]);
}
