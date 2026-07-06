use crate::{ProviderHttpConfig, ProviderSettings, SessionConfig};
use versionlens_parsers::Ecosystem::{Cargo, Npm};
use versionlens_parsers::ManifestKind::{NpmPackageJson, PnpmYaml};

#[test]
fn provider_http_overrides_global_strict_ssl() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            provider_http: vec![
                ProviderHttpConfig {
                    ecosystem: Npm,
                    manifest_kind: None,
                    strict_ssl: Some(false),
                },
                ProviderHttpConfig {
                    ecosystem: Npm,
                    manifest_kind: None,
                    strict_ssl: Some(true),
                },
            ],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    assert!(session.http_config(Npm).strict_ssl);
    assert!(session.http_config(Cargo).strict_ssl);
}

#[test]
fn manifest_scoped_provider_http_does_not_override_package_json_npm() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            provider_http: vec![
                ProviderHttpConfig {
                    ecosystem: Npm,
                    manifest_kind: None,
                    strict_ssl: Some(false),
                },
                ProviderHttpConfig {
                    ecosystem: Npm,
                    manifest_kind: Some(PnpmYaml),
                    strict_ssl: Some(true),
                },
            ],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    assert!(!session.http_config(Npm).strict_ssl);
    assert!(
        session
            .http_config_for_manifest(Npm, Some(PnpmYaml))
            .strict_ssl
    );
    assert!(
        !session
            .http_config_for_manifest(Npm, Some(NpmPackageJson))
            .strict_ssl
    );
}
