use super::ApplyCommandRequest;
use versionlens_parsers::{DocumentInput, Ecosystem};

use crate::{
    DependencyPropertyConfig, ProviderSettings, RegistryResponseInput, SessionConfig,
    VersionLensSession,
};

fn standard_session() -> VersionLensSession {
    session_with_vulnerability_visibility(true)
}

fn session_with_vulnerability_visibility(show_vulnerabilities: bool) -> VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}

fn session_with_dependency_properties(
    ecosystem: Ecosystem,
    properties: &[&str],
) -> VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            dependency_properties: vec![DependencyPropertyConfig {
                ecosystem,
                manifest_kind: None,
                properties: properties
                    .iter()
                    .map(|property| (*property).to_owned())
                    .collect(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}

mod sort;
mod update;
