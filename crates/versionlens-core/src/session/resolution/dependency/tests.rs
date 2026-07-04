use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem};

use crate::{
    ProviderSettings, RegistryResponseInput, SessionConfig, SuggestionIndicators,
    VersionLensSession,
};

#[test]
fn resolves_repo_versions_as_current_when_registry_matches() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///dub.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"vibe-d":"~master"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "vibe-d".to_owned(),
            ecosystem: Ecosystem::Dub,
            body: r#"{"versions":[{"version":"~master"},{"version":"0.9.0"},{"version":"0.8.0"}]}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].latest.as_deref(), Some("~master"));
    assert_eq!(output.suggestions[0].status, "current");
    assert!(output.edits.is_empty());
}

#[test]
fn pub_hosted_git_dependencies_are_fixed_without_git_suffix() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let output = session.resolve_document(DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "dependencies:\n  repo:\n    git:\n      url: https://github.com/owner/repo\n"
            .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.suggestions[0].dependency.name, "repo");
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
}
