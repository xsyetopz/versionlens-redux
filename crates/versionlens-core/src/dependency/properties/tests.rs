use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem, ManifestKind};

use crate::{
    DependencyPropertyConfig, ProviderSettings, SessionConfig, SuggestionIndicators,
    VersionLensSession,
};

fn session_with_properties(ecosystem: Ecosystem, properties: &[&str]) -> VersionLensSession {
    session_with_property_configs(&[(ecosystem, properties)])
}

fn session_with_property_configs(configs: &[(Ecosystem, &[&str])]) -> VersionLensSession {
    session_with_scoped_property_configs(
        &configs
            .iter()
            .map(|(ecosystem, properties)| (*ecosystem, None, *properties))
            .collect::<Vec<_>>(),
    )
}

fn session_with_scoped_property_configs(
    configs: &[(Ecosystem, Option<ManifestKind>, &[&str])],
) -> VersionLensSession {
    VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            dependency_properties: configs
                .iter()
                .map(
                    |(ecosystem, manifest_kind, properties)| DependencyPropertyConfig {
                        ecosystem: *ecosystem,
                        manifest_kind: *manifest_kind,
                        properties: properties
                            .iter()
                            .map(|property| (*property).to_owned())
                            .collect(),
                    },
                )
                .collect(),
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    })
}

mod cargo;
mod npm;
mod pub_manifest;
mod python;
mod xml;
