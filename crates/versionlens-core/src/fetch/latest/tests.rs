use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem};

use crate::{
    ProviderSettings, RegistryUrlConfig, SessionConfig, SuggestionIndicators, VersionLensSession,
};

#[test]
fn invalid_registry_url_creates_contextual_error_suggestion() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Ecosystem::Npm,
                url: "not a url".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.resolve_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
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
