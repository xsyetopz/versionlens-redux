use versionlens_http::HttpConfig;
use versionlens_parsers::{Ecosystem, ManifestKind};

use crate::{
    ProviderHttpConfig, ProviderSettings, SessionConfig, SuggestionIndicators, VersionLensSession,
};

#[test]
fn provider_http_overrides_global_strict_ssl() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            provider_http: vec![
                ProviderHttpConfig {
                    ecosystem: Ecosystem::Npm,
                    manifest_kind: None,
                    strict_ssl: Some(false),
                },
                ProviderHttpConfig {
                    ecosystem: Ecosystem::Npm,
                    manifest_kind: None,
                    strict_ssl: Some(true),
                },
            ],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    assert!(session.http_config(Ecosystem::Npm).strict_ssl);
    assert!(session.http_config(Ecosystem::Cargo).strict_ssl);
}

#[test]
fn manifest_scoped_provider_http_does_not_override_package_json_npm() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            provider_http: vec![
                ProviderHttpConfig {
                    ecosystem: Ecosystem::Npm,
                    manifest_kind: None,
                    strict_ssl: Some(false),
                },
                ProviderHttpConfig {
                    ecosystem: Ecosystem::Npm,
                    manifest_kind: Some(ManifestKind::PnpmYaml),
                    strict_ssl: Some(true),
                },
            ],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    assert!(!session.http_config(Ecosystem::Npm).strict_ssl);
    assert!(
        session
            .http_config_for_manifest(Ecosystem::Npm, Some(ManifestKind::PnpmYaml))
            .strict_ssl
    );
    assert!(
        !session
            .http_config_for_manifest(Ecosystem::Npm, Some(ManifestKind::NpmPackageJson))
            .strict_ssl
    );
}
