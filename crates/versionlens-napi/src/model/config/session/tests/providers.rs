use super::blank_session_config;
use crate::model::config::{
    NativeDependencyPropertyConfig, NativeFilePatternConfig, NativeProviderCacheConfig,
    NativeProviderHttpConfig, NativeProviderSettings, NativeRegistryUrl, NativeSessionConfig,
};
use versionlens_parsers::Ecosystem::{Dotnet, Npm};
use versionlens_parsers::ManifestKind::{ComposerJson, PnpmYaml};

#[test]
fn pnpm_config_namespace_maps_to_npm_provider() {
    let config = NativeSessionConfig {
        enabled_providers: Some(vec!["pnpm".to_owned()]),
        providers: Some(NativeProviderSettings {
            registry_urls: None,
            prerelease_tag_filters: None,
            provider_cache: Some(vec![NativeProviderCacheConfig {
                ecosystem: "pnpm".to_owned(),
                cache_duration_minutes: Some(2.0),
            }]),
            provider_http: Some(vec![NativeProviderHttpConfig {
                ecosystem: "pnpm".to_owned(),
                strict_ssl: Some(false),
            }]),
            dependency_properties: Some(vec![NativeDependencyPropertyConfig {
                provider: None,
                ecosystem: "pnpm".to_owned(),
                properties: vec!["catalog".to_owned()],
            }]),
            file_patterns: None,
        }),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(config.enabled_providers[0].ecosystem, Npm);
    assert_eq!(config.enabled_providers[0].manifest_kind, Some(PnpmYaml));
    assert_eq!(config.providers.provider_cache[0].ecosystem, Npm);
    assert_eq!(
        config.providers.provider_cache[0].manifest_kind,
        Some(PnpmYaml)
    );
    assert_eq!(config.providers.provider_cache[0].cache_ttl_ms, 120_000);
    assert_eq!(config.providers.provider_http[0].ecosystem, Npm);
    assert_eq!(
        config.providers.provider_http[0].manifest_kind,
        Some(PnpmYaml)
    );
    assert_eq!(config.providers.provider_http[0].strict_ssl, Some(false));
    assert_eq!(config.providers.dependency_properties[0].ecosystem, Npm);
    assert_eq!(
        config.providers.dependency_properties[0].manifest_kind,
        Some(PnpmYaml)
    );
}

#[test]
fn provider_fractional_cache_minutes_are_converted_to_milliseconds() {
    let config = NativeSessionConfig {
        providers: Some(NativeProviderSettings {
            registry_urls: None,
            prerelease_tag_filters: None,
            provider_cache: Some(vec![NativeProviderCacheConfig {
                ecosystem: "npm".to_owned(),
                cache_duration_minutes: Some(0.25),
            }]),
            provider_http: None,
            dependency_properties: None,
            file_patterns: None,
        }),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(config.providers.provider_cache[0].cache_ttl_ms, 15_000);
}

#[test]
fn registry_urls_are_trimmed_and_blank_urls_are_ignored_in_rust() {
    let config = NativeSessionConfig {
        providers: Some(NativeProviderSettings {
            registry_urls: Some(vec![
                NativeRegistryUrl {
                    ecosystem: "cargo".to_owned(),
                    url: "   ".to_owned(),
                },
                NativeRegistryUrl {
                    ecosystem: "cargo".to_owned(),
                    url: " https://mirror.test/crates ".to_owned(),
                },
                NativeRegistryUrl {
                    ecosystem: "dotnet".to_owned(),
                    url: " https://configured.nuget/v3/index.json ".to_owned(),
                },
            ]),
            prerelease_tag_filters: None,
            provider_cache: None,
            provider_http: None,
            dependency_properties: None,
            file_patterns: None,
        }),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(config.providers.registry_urls.len(), 2);
    assert_eq!(
        config.providers.registry_urls[0].url,
        "https://mirror.test/crates"
    );
    assert_eq!(
        config
            .providers
            .registry_urls
            .iter()
            .filter(|url| url.ecosystem == Dotnet)
            .map(|url| url.url.as_str())
            .collect::<Vec<_>>(),
        ["https://configured.nuget/v3/index.json"]
    );
}

#[test]
fn file_patterns_are_trimmed_and_mapped_to_manifest_kinds() {
    let config = NativeSessionConfig {
        providers: Some(NativeProviderSettings {
            registry_urls: None,
            prerelease_tag_filters: None,
            provider_cache: None,
            provider_http: None,
            dependency_properties: None,
            file_patterns: Some(vec![
                NativeFilePatternConfig {
                    ecosystem: "composer".to_owned(),
                    pattern: " **/acme.composer.json ".to_owned(),
                },
                NativeFilePatternConfig {
                    ecosystem: "composer".to_owned(),
                    pattern: " ".to_owned(),
                },
            ]),
        }),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(config.providers.file_patterns.len(), 1);
    assert_eq!(
        config.providers.file_patterns[0].manifest_kind,
        ComposerJson
    );
    assert_eq!(
        config.providers.file_patterns[0].pattern,
        "**/acme.composer.json"
    );
}
