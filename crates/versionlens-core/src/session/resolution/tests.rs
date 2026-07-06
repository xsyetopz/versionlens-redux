use versionlens_parsers::{DocumentInput, parse_document};

use crate::{
    ProviderSettings, RegistryResponseInput, RegistryUrlConfig, SessionConfig, VersionLensSession,
};

fn standard_session() -> VersionLensSession {
    session_with_settings(crate::default(), true)
}

fn session_without_vulnerabilities() -> VersionLensSession {
    session_with_settings(crate::default(), false)
}

fn session_with_settings(
    providers: ProviderSettings,
    show_vulnerabilities: bool,
) -> VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers,
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}

mod dotnet;
mod error;
mod fixed;
mod github;
mod go;
mod hex;
mod maven;
mod npm;
mod npm_request_cache;
mod project;
