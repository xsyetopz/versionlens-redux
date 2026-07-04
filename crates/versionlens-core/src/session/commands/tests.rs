use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem};

use crate::{
    DependencyPropertyConfig, ProviderSettings, RegistryResponseInput, SessionConfig,
    SuggestionIndicators, VersionLensSession,
};

fn standard_session() -> VersionLensSession {
    session_with_vulnerability_visibility(true)
}

fn session_with_vulnerability_visibility(show_vulnerabilities: bool) -> VersionLensSession {
    VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    })
}

fn session_with_dependency_properties(
    ecosystem: Ecosystem,
    properties: &[&str],
) -> VersionLensSession {
    VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            dependency_properties: vec![DependencyPropertyConfig {
                ecosystem,
                manifest_kind: None,
                properties: properties
                    .iter()
                    .map(|property| (*property).to_owned())
                    .collect(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    })
}

mod sort;
mod update;
