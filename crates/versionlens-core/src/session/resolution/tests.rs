use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem, parse_document};

use crate::{
    DependencyPropertyConfig, ProviderSettings, RegistryResponseInput, RegistryUrlConfig,
    SessionConfig, SuggestionIndicators, VersionLensSession,
};

fn standard_session() -> VersionLensSession {
    session_with_settings(ProviderSettings::default(), true)
}

fn session_without_vulnerabilities() -> VersionLensSession {
    session_with_settings(ProviderSettings::default(), false)
}

fn session_with_dependency_properties(
    ecosystem: Ecosystem,
    properties: &[&str],
) -> VersionLensSession {
    session_with_settings(
        ProviderSettings {
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
        true,
    )
}

fn session_with_settings(
    providers: ProviderSettings,
    show_vulnerabilities: bool,
) -> VersionLensSession {
    VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers,
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    })
}

mod dotnet;
mod error;
mod fixed;
mod github;
mod go;
mod maven;
mod npm;
mod npm_request_cache;
mod project;
