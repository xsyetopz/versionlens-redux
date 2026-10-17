//! The `versionlens` settings section, in the key shape that the VS Code
//! extension contributes, so every editor configures the server the same way.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};
use versionlens_fetch::HttpConfigInput;

use super::{
    DependencyPropertyConfigInput, FilePatternConfigInput, PrereleaseTagConfigInput,
    ProviderCacheConfigInput, ProviderHttpConfigInput, ProviderSettingsInput,
    RegistryUrlConfigInput, SessionConfig, SessionConfigInput, SuggestionIndicatorsInput,
    UpdateSeverity,
};

/// The `versionlens` settings section. A missing key keeps the engine default.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub enabled_providers: Option<Vec<String>>,
    pub caching: GlobalCaching,
    pub http: HttpSettings,
    pub suggestions: SuggestionSettings,
    pub diagnostics: DiagnosticSettings,
    /// Settings per ecosystem, keyed by its configuration name, such as
    /// `npm` or `dotnet`.
    #[serde(flatten)]
    pub ecosystems: BTreeMap<String, EcosystemSettings>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct GlobalCaching {
    /// Minutes. `null` turns caching off, and a missing key keeps the
    /// default.
    #[serde(deserialize_with = "null_is_zero")]
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct EcosystemCaching {
    /// Minutes. `null` or a missing key uses the global duration.
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct HttpSettings {
    #[serde(rename = "strictSSL")]
    pub strict_ssl: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DiagnosticSettings {
    pub update_severity: Option<UpdateSeverity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SuggestionSettings {
    pub show_prereleases_on_startup: Option<bool>,
    pub show_vulnerabilities: Option<bool>,
    pub show_suggestions_stats: Option<bool>,
    /// Indicator text by name. Both the `camelCase` names and the `PascalCase`
    /// names of the contributed default are read.
    pub indicators: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EcosystemSettings {
    pub files: Option<String>,
    pub prerelease_tag_filter: Option<Vec<String>>,
    pub dependency_properties: Option<Vec<String>>,
    pub api_url: Option<String>,
    pub caching: EcosystemCaching,
    pub http: HttpSettings,
    pub nuget: NugetSettings,
    /// The client runs this command, so the server only accepts the key.
    pub on_save_changes: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct NugetSettings {
    pub sources: Option<Vec<String>>,
}

/// Reads `null` as zero minutes, so `null` turns caching off while a missing
/// key keeps the default.
fn null_is_zero<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<f64>::deserialize(deserializer).map(|minutes| Some(minutes.unwrap_or(0.0)))
}

/// The `camelCase` name of each indicator and the `PascalCase` name in the
/// contributed default.
const INDICATOR_NAMES: [(&str, &str); 9] = [
    ("latest", "Latest"),
    ("satisfiesLatest", "SatisfiesLatest"),
    ("directory", "Directory"),
    ("error", "Error"),
    ("noMatch", "NoMatch"),
    ("matched", "Match"),
    ("updateable", "Updateable"),
    ("updateableVulnerable", "UpdateableVulnerable"),
    ("build", "Build"),
];

impl Settings {
    /// Reads a settings section. A client answers `null` for a section it
    /// does not have, which keeps every default.
    pub fn from_value(value: serde_json::Value) -> Result<Self, String> {
        if value.is_null() {
            return Ok(Self::default());
        }
        serde_json::from_value(value).map_err(|error| error.to_string())
    }

    pub fn session_config(self) -> SessionConfig {
        SessionConfig::from(self.into_input())
    }

    fn into_input(self) -> SessionConfigInput {
        let mut providers = ProviderSettingsInput {
            registry_urls: Some(Vec::new()),
            prerelease_tag_filters: Some(Vec::new()),
            provider_cache: Some(Vec::new()),
            provider_http: Some(Vec::new()),
            dependency_properties: Some(Vec::new()),
            file_patterns: Some(Vec::new()),
        };
        for (name, settings) in self.ecosystems {
            settings.add_to(&name, &mut providers);
        }
        SessionConfigInput {
            cache_duration_minutes: self.caching.duration,
            cache_ttl_seconds: None,
            enabled_providers: self.enabled_providers,
            providers: Some(providers),
            suggestion_indicators: self.suggestions.indicators.map(indicators),
            show_vulnerabilities: self.suggestions.show_vulnerabilities,
            show_suggestion_stats: self.suggestions.show_suggestions_stats,
            show_prereleases: self
                .suggestions
                .show_prereleases_on_startup
                .unwrap_or(false),
            update_severity: self.diagnostics.update_severity,
            http: Some(HttpConfigInput {
                timeout_ms: None,
                strict_ssl: self.http.strict_ssl,
                proxy: None,
                ca_file: None,
                ca: None,
                cert_file: None,
                key_file: None,
                cert: None,
                key: None,
                auth_headers: None,
            }),
        }
    }
}

impl EcosystemSettings {
    fn add_to(self, name: &str, providers: &mut ProviderSettingsInput) {
        let ecosystem = || name.to_owned();
        let registry_urls = providers.registry_urls.get_or_insert_default();
        registry_urls.extend(
            self.api_url
                .into_iter()
                .chain(self.nuget.sources.into_iter().flatten())
                .map(|url| RegistryUrlConfigInput {
                    ecosystem: ecosystem(),
                    url,
                }),
        );
        if let Some(pattern) = self.files {
            providers
                .file_patterns
                .get_or_insert_default()
                .push(FilePatternConfigInput {
                    ecosystem: ecosystem(),
                    pattern,
                });
        }
        if let Some(tags) = self.prerelease_tag_filter {
            providers
                .prerelease_tag_filters
                .get_or_insert_default()
                .push(PrereleaseTagConfigInput {
                    ecosystem: ecosystem(),
                    tags,
                });
        }
        if let Some(properties) = self.dependency_properties {
            providers
                .dependency_properties
                .get_or_insert_default()
                .push(DependencyPropertyConfigInput {
                    ecosystem: ecosystem(),
                    provider: None,
                    properties,
                });
        }
        if let Some(minutes) = self.caching.duration {
            providers
                .provider_cache
                .get_or_insert_default()
                .push(ProviderCacheConfigInput {
                    ecosystem: ecosystem(),
                    cache_duration_minutes: Some(minutes),
                });
        }
        if let Some(strict_ssl) = self.http.strict_ssl {
            providers
                .provider_http
                .get_or_insert_default()
                .push(ProviderHttpConfigInput {
                    ecosystem: ecosystem(),
                    strict_ssl: Some(strict_ssl),
                });
        }
    }
}

fn indicators(mut configured: BTreeMap<String, String>) -> SuggestionIndicatorsInput {
    let mut take = |index: usize| {
        let (name, legacy) = INDICATOR_NAMES[index];
        let legacy = configured.remove(legacy);
        configured.remove(name).or(legacy)
    };
    SuggestionIndicatorsInput {
        latest: take(0),
        satisfies_latest: take(1),
        directory: take(2),
        error: take(3),
        no_match: take(4),
        matched: take(5),
        updateable: take(6),
        updateable_vulnerable: take(7),
        build: take(8),
    }
}

#[cfg(test)]
mod tests;
