use versionlens_parsers::{Ecosystem, ManifestKind};

use super::blank_session_config;
use versionlens_core::SessionConfig;

use crate::model::config::{
    NativeDependencyPropertyConfig, NativeFilePatternConfig, NativeHttpConfig, NativeHttpHeader,
    NativePrereleaseTagFilter, NativeProviderCacheConfig, NativeProviderHttpConfig,
    NativeProviderSettings, NativeRegistryUrl, NativeSuggestionIndicators,
};

#[test]
fn maps_session_http_config_to_core() {
    let config = mapped_session_config();

    assert_session_flags(&config);
    assert_provider_config(&config);
    assert_suggestion_indicators(&config);
    assert_http_config(&config);
}

fn mapped_session_config() -> SessionConfig {
    crate::model::config::NativeSessionConfig {
        cache_duration_minutes: Some(2.0),
        enabled_providers: Some(vec![
            "cargo".to_owned(),
            "golang".to_owned(),
            "pypi".to_owned(),
        ]),
        providers: Some(NativeProviderSettings {
            registry_urls: Some(vec![NativeRegistryUrl {
                ecosystem: "cargo".to_owned(),
                url: "https://mirror.test/crates".to_owned(),
            }]),
            prerelease_tag_filters: Some(vec![NativePrereleaseTagFilter {
                ecosystem: "npm".to_owned(),
                tags: vec![" beta ".to_owned(), String::new(), "  ".to_owned()],
            }]),
            provider_cache: Some(vec![NativeProviderCacheConfig {
                ecosystem: "npm".to_owned(),
                cache_duration_minutes: Some(1.0),
            }]),
            provider_http: Some(vec![NativeProviderHttpConfig {
                ecosystem: "npm".to_owned(),
                strict_ssl: Some(false),
            }]),
            dependency_properties: Some(vec![NativeDependencyPropertyConfig {
                provider: None,
                ecosystem: "npm".to_owned(),
                properties: vec!["dependencies".to_owned()],
            }]),
            file_patterns: Some(vec![NativeFilePatternConfig {
                ecosystem: "composer".to_owned(),
                pattern: " **/acme.composer.json ".to_owned(),
            }]),
        }),
        suggestion_indicators: Some(NativeSuggestionIndicators {
            build: Some("B".to_owned()),
            latest: Some("L".to_owned()),
            satisfies_latest: Some("S".to_owned()),
            directory: Some("D".to_owned()),
            error: Some("E".to_owned()),
            no_match: Some("N".to_owned()),
            matched: Some("M".to_owned()),
            updateable: Some("U".to_owned()),
            updateable_vulnerable: Some("V".to_owned()),
        }),
        show_vulnerabilities: Some(false),
        show_suggestion_stats: Some(true),
        show_prereleases: true,
        http: Some(NativeHttpConfig {
            timeout_ms: Some(123),
            strict_ssl: Some(false),
            ca_file: Some(" /tmp/native-ca.pem ".to_owned()),
            ca: None,
            cert_file: None,
            key_file: None,
            cert: None,
            key: None,
            proxy: Some("http://localhost:8080".to_owned()),
            auth_headers: Some(vec![NativeHttpHeader {
                name: "authorization".to_owned(),
                value: "Bearer token".to_owned(),
                url: Some("https://registry.example.test".to_owned()),
            }]),
        }),
    }
    .into_core()
}

fn assert_session_flags(config: &SessionConfig) {
    assert!(config.show_prereleases);
    assert!(!config.show_vulnerabilities);
    assert!(config.show_suggestion_stats);
    assert_eq!(
        config
            .enabled_providers
            .iter()
            .map(|provider| provider.ecosystem)
            .collect::<Vec<_>>(),
        [Ecosystem::Cargo, Ecosystem::Go, Ecosystem::Python]
    );
    assert_eq!(config.cache_ttl_ms, 120_000);
}

fn assert_provider_config(config: &SessionConfig) {
    assert_eq!(
        config.providers.registry_urls[0].ecosystem,
        Ecosystem::Cargo
    );
    assert_eq!(
        config.providers.registry_urls[0].url,
        "https://mirror.test/crates"
    );
    assert_eq!(
        config.providers.prerelease_tags[0].ecosystem,
        Ecosystem::Npm
    );
    assert_eq!(config.providers.prerelease_tags[0].tags, ["beta"]);
    assert_eq!(config.providers.provider_cache[0].ecosystem, Ecosystem::Npm);
    assert_eq!(config.providers.provider_cache[0].manifest_kind, None);
    assert_eq!(config.providers.provider_cache[0].cache_ttl_ms, 60_000);
    assert_eq!(config.providers.provider_http[0].ecosystem, Ecosystem::Npm);
    assert_eq!(config.providers.provider_http[0].manifest_kind, None);
    assert_eq!(config.providers.provider_http[0].strict_ssl, Some(false));
    assert_eq!(
        config.providers.dependency_properties[0].ecosystem,
        Ecosystem::Npm
    );
    assert_eq!(
        config.providers.dependency_properties[0].properties,
        ["dependencies"]
    );
    assert_eq!(
        config.providers.dependency_properties[0].manifest_kind,
        Some(ManifestKind::NpmPackageJson)
    );
    assert_eq!(
        config.providers.file_patterns[0].manifest_kind,
        ManifestKind::ComposerJson
    );
    assert_eq!(
        config.providers.file_patterns[0].pattern,
        "**/acme.composer.json"
    );
}

fn assert_suggestion_indicators(config: &SessionConfig) {
    assert_eq!(config.suggestion_indicators.latest, "L");
    assert_eq!(config.suggestion_indicators.directory, "D");
    assert_eq!(config.suggestion_indicators.error, "E");
    assert_eq!(config.suggestion_indicators.matched, "M");
    assert_eq!(config.suggestion_indicators.build, "B");
    assert_eq!(config.suggestion_indicators.updateable_vulnerable, "V");
}

fn assert_http_config(config: &SessionConfig) {
    assert_eq!(config.http.timeout_ms, 123);
    assert!(!config.http.strict_ssl);
    assert_eq!(config.http.proxy.as_deref(), Some("http://localhost:8080"));
    assert_eq!(config.http.ca_file.as_deref(), Some("/tmp/native-ca.pem"));
    assert_eq!(config.http.auth_headers[0].name, "authorization");
    assert_eq!(
        config.http.auth_headers[0].url.as_deref(),
        Some("https://registry.example.test")
    );
}

#[test]
fn missing_flags_use_defaults() {
    let config = blank_session_config().into_core();

    assert!(!config.show_prereleases);
    assert!(config.show_vulnerabilities);
    assert!(!config.show_suggestion_stats);
}
