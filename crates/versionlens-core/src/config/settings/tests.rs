use serde_json::json;
use versionlens_model::{Ecosystem, ManifestKind};

use super::Settings;
use crate::SessionConfig;

fn config(value: serde_json::Value) -> SessionConfig {
    Settings::from_value(value)
        .expect("valid settings")
        .session_config()
}

#[test]
fn missing_settings_give_the_engine_defaults() {
    assert_eq!(config(json!({})), SessionConfig::default());
    assert_eq!(config(json!(null)), SessionConfig::default());
}

#[test]
fn vscode_keys_map_onto_the_session() {
    let config = config(json!({
        "enabledProviders": ["npm"],
        "caching": {"duration": 2},
        "http": {"strictSSL": false},
        "suggestions": {
            "showPrereleasesOnStartup": true,
            "showVulnerabilities": false,
            "showSuggestionsStats": true,
            "indicators": {"Latest": "L", "noMatch": "N"}
        },
        "npm": {
            "apiUrl": "http://registry.test",
            "prereleaseTagFilter": ["beta"],
            "dependencyProperties": ["dependencies"],
            "files": "**/package.json",
            "caching": {"duration": 7},
            "http": {"strictSSL": true},
            "onSaveChanges": "npm install"
        },
        "dotnet": {"nuget": {"sources": ["http://nuget.test/v3/index.json"]}},
        "pnpm": {"dependencyProperties": ["catalog"]}
    }));
    assert_eq!(config.cache_ttl_ms, 120_000);
    assert_eq!(config.enabled_providers.len(), 1);
    assert!(!config.http.strict_ssl);
    assert!(config.show_prereleases);
    assert!(!config.show_vulnerabilities);
    assert!(config.show_suggestion_stats);
    assert_eq!(config.suggestion_indicators.latest, "L");
    assert_eq!(config.suggestion_indicators.no_match, "N");

    let providers = &config.providers;
    let urls = providers
        .registry_urls
        .iter()
        .map(|url| (url.ecosystem, url.url.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            (Ecosystem::Dotnet, "http://nuget.test/v3/index.json"),
            (Ecosystem::Npm, "http://registry.test"),
        ]
    );
    assert_eq!(providers.prerelease_tags[0].tags, ["beta"]);
    assert_eq!(providers.provider_cache[0].cache_ttl_ms, 420_000);
    assert_eq!(providers.provider_http[0].ecosystem, Ecosystem::Npm);
    assert!(providers.file_patterns.iter().any(|pattern| {
        pattern.manifest_kind == ManifestKind::NpmPackageJson
            && pattern.pattern == "**/package.json"
    }));
    let pnpm = providers
        .dependency_properties
        .iter()
        .find(|config| config.manifest_kind == Some(ManifestKind::PnpmYaml))
        .expect("pnpm properties");
    assert_eq!(pnpm.properties, ["catalog"]);
}

#[test]
fn a_wrongly_typed_value_is_an_error() {
    assert!(Settings::from_value(json!({"caching": {"duration": "soon"}})).is_err());
    assert!(Settings::from_value(json!({"npm": {"files": 3}})).is_err());
}
