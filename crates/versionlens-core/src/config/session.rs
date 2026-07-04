use serde::{Deserialize, Serialize};
use versionlens_cache::cache_ttl_ms;
use versionlens_http::{HttpConfig, HttpConfigInput};

use super::{
    EnabledProviderConfig, ProviderSettings, ProviderSettingsInput, SuggestionIndicators,
    SuggestionIndicatorsInput, enabled_provider_config_from_name,
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
                .map(ProviderSettings::from_input)
                .unwrap_or_default(),
            suggestion_indicators: input
                .suggestion_indicators
                .map(SuggestionIndicators::from_input)
                .unwrap_or_else(SuggestionIndicators::standard),
            show_vulnerabilities: input.show_vulnerabilities.unwrap_or(true),
            show_suggestion_stats: input.show_suggestion_stats.unwrap_or(false),
            show_prereleases: input.show_prereleases,
            http: input
                .http
                .map(HttpConfig::from_input)
                .unwrap_or_else(HttpConfig::standard),
        }
    }
}
