//! The `versionlens` settings section, in the key shape that the VS Code
//! extension contributes, so every editor configures the server the same way.

use std::collections::BTreeMap;

use serde::Deserialize;
use versionlens_fetch::{HttpConfig, cache_ttl_ms, standard_http_config};

use super::{
    ProviderSettings, SessionConfig, SuggestionIndicators, UpdateSeverity,
    dependency_property_config_from_name, enabled_provider_config_from_name,
    file_pattern_config_from_name, prerelease_tag_config_from_name,
    provider_cache_config_from_name, provider_http_config_from_name, registry_url_config_from_name,
    standard_suggestion_indicators,
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
    /// Read by the language server, which decides what to scan.
    pub workspace: WorkspaceSettings,
    /// Settings per ecosystem, keyed by its configuration name, such as
    /// `npm` or `dotnet`.
    #[serde(flatten)]
    pub ecosystems: BTreeMap<String, EcosystemSettings>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkspaceSettings {
    pub check: WorkspaceCheck,
    /// Glob patterns of workspace paths never checked, on top of the
    /// ignore files and `files.exclude`.
    pub exclude: Vec<String>,
}

/// Which manifests are checked in the background.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceCheck {
    #[default]
    OpenFiles,
    Workspace,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct GlobalCaching {
    /// Minutes. Zero turns caching off.
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
    /// Indicator text by `camelCase` name.
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
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct NugetSettings {
    pub sources: Option<Vec<String>>,
}

impl Settings {
    /// Reads a settings section. A client answers `null` for a section it
    /// does not have, which keeps every default.
    pub fn from_value(value: serde_json::Value) -> Result<Self, String> {
        if value.is_null() {
            return Ok(Self::default());
        }
        serde_json::from_value(value).map_err(|error| error.to_string())
    }

    /// The engine settings. The workspace settings are not part of them.
    pub fn session_config(self) -> SessionConfig {
        let mut providers = ProviderSettings::default();
        for (name, settings) in self.ecosystems {
            settings.add_to(&name, &mut providers);
        }
        SessionConfig {
            cache_ttl_ms: cache_ttl_ms(self.caching.duration, None),
            enabled_providers: self
                .enabled_providers
                .unwrap_or_default()
                .iter()
                .filter_map(|provider| enabled_provider_config_from_name(provider))
                .collect(),
            providers,
            suggestion_indicators: self
                .suggestions
                .indicators
                .map(indicators)
                .unwrap_or_else(standard_suggestion_indicators),
            show_vulnerabilities: self.suggestions.show_vulnerabilities.unwrap_or(true),
            show_suggestion_stats: self.suggestions.show_suggestions_stats.unwrap_or(false),
            show_prereleases: self
                .suggestions
                .show_prereleases_on_startup
                .unwrap_or(false),
            update_severity: self
                .diagnostics
                .update_severity
                .unwrap_or_default()
                .severity(),
            http: HttpConfig {
                strict_ssl: self.http.strict_ssl.unwrap_or(true),
                ..standard_http_config()
            },
        }
    }
}

impl EcosystemSettings {
    fn add_to(self, name: &str, providers: &mut ProviderSettings) {
        providers.registry_urls.extend(
            self.api_url
                .into_iter()
                .chain(self.nuget.sources.into_iter().flatten())
                .filter_map(|url| registry_url_config_from_name(name, url)),
        );
        providers.file_patterns.extend(
            self.files
                .and_then(|pattern| file_pattern_config_from_name(name, pattern)),
        );
        providers.prerelease_tags.extend(
            self.prerelease_tag_filter
                .and_then(|tags| prerelease_tag_config_from_name(name, tags)),
        );
        providers.dependency_properties.extend(
            self.dependency_properties.and_then(|properties| {
                dependency_property_config_from_name(name, None, properties)
            }),
        );
        providers
            .provider_cache
            .extend(provider_cache_config_from_name(name, self.caching.duration));
        providers.provider_http.extend(
            self.http
                .strict_ssl
                .and_then(|strict_ssl| provider_http_config_from_name(name, Some(strict_ssl))),
        );
    }
}

fn indicators(mut configured: BTreeMap<String, String>) -> SuggestionIndicators {
    let mut take = |name: &str| configured.remove(name).unwrap_or_default();
    SuggestionIndicators {
        latest: take("latest"),
        satisfies_latest: take("satisfiesLatest"),
        directory: take("directory"),
        error: take("error"),
        no_match: take("noMatch"),
        matched: take("matched"),
        updateable: take("updateable"),
        updateable_vulnerable: take("updateableVulnerable"),
        build: take("build"),
    }
    .with_standard_indicators_for_blanks()
}

#[cfg(test)]
mod tests;
