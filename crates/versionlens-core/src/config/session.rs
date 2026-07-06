use serde::{Deserialize, Serialize};
use versionlens_cache::cache_ttl_ms;
use versionlens_http::{HttpConfig, HttpConfigInput, standard_http_config};

use super::{
    EnabledProviderConfig, ProviderSettings, ProviderSettingsInput, SuggestionIndicators,
    SuggestionIndicatorsInput, enabled_provider_config_from_name, standard_suggestion_indicators,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    pub cache_ttl_ms: u64,
    pub enabled_providers: Vec<EnabledProviderConfig>,
    pub providers: ProviderSettings,
    pub suggestion_indicators: SuggestionIndicators,
    pub show_vulnerabilities: bool,
    pub show_suggestion_stats: bool,
    pub show_prereleases: bool,
    pub http: HttpConfig,
}

#[derive(Debug, Default, PartialEq)]
pub struct SessionConfigInput {
    pub cache_duration_minutes: Option<f64>,
    pub cache_ttl_seconds: Option<u32>,
    pub enabled_providers: Option<Vec<String>>,
    pub providers: Option<ProviderSettingsInput>,
    pub suggestion_indicators: Option<SuggestionIndicatorsInput>,
    pub show_vulnerabilities: Option<bool>,
    pub show_suggestion_stats: Option<bool>,
    pub show_prereleases: bool,
    pub http: Option<HttpConfigInput>,
}

fn provider_settings_from_input(input: ProviderSettingsInput) -> ProviderSettings {
    input.into()
}

fn suggestion_indicators_from_input(input: SuggestionIndicatorsInput) -> SuggestionIndicators {
    input.into()
}

fn http_config_from_input(input: HttpConfigInput) -> HttpConfig {
    versionlens_http::http_config_from_input(input)
}

impl SessionConfig {
    pub fn from_input(input: SessionConfigInput) -> Self {
        Self {
            cache_ttl_ms: cache_ttl_ms(input.cache_duration_minutes, input.cache_ttl_seconds),
            enabled_providers: input
                .enabled_providers
                .unwrap_or_default()
                .into_iter()
                .filter_map(|provider| enabled_provider_config_from_name(&provider))
                .collect(),
            providers: input
                .providers
                .map(provider_settings_from_input)
                .unwrap_or_default(),
            suggestion_indicators: input
                .suggestion_indicators
                .map(suggestion_indicators_from_input)
                .unwrap_or_else(standard_suggestion_indicators),
            show_vulnerabilities: input.show_vulnerabilities.unwrap_or(true),
            show_suggestion_stats: input.show_suggestion_stats.unwrap_or(false),
            show_prereleases: input.show_prereleases,
            http: input
                .http
                .map(http_config_from_input)
                .unwrap_or_else(standard_http_config),
        }
    }
}

impl From<SessionConfigInput> for SessionConfig {
    fn from(input: SessionConfigInput) -> Self {
        Self::from_input(input)
    }
}

impl From<ProviderSettingsInput> for ProviderSettings {
    fn from(input: ProviderSettingsInput) -> Self {
        Self::from_input(input)
    }
}

impl From<SuggestionIndicatorsInput> for SuggestionIndicators {
    fn from(input: SuggestionIndicatorsInput) -> Self {
        Self::from_input(input)
    }
}
