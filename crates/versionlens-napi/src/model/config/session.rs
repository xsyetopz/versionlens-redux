use napi_derive::napi;
use versionlens_core::{SessionConfig, SessionConfigInput};

use super::http::NativeHttpConfig;
use super::providers::NativeProviderSettings;
use super::suggestions::NativeSuggestionIndicators;

#[napi(object)]
pub struct NativeSessionConfig {
    pub cache_duration_minutes: Option<f64>,
    pub enabled_providers: Option<Vec<String>>,
    pub providers: Option<NativeProviderSettings>,
    pub suggestion_indicators: Option<NativeSuggestionIndicators>,
    pub show_vulnerabilities: Option<bool>,
    pub show_suggestion_stats: Option<bool>,
    pub show_prereleases: bool,
    pub http: Option<NativeHttpConfig>,
}

impl NativeSessionConfig {
    pub(crate) fn into_core(self) -> SessionConfig {
        SessionConfig::from_input(SessionConfigInput {
            cache_duration_minutes: self.cache_duration_minutes,
            cache_ttl_seconds: None,
            enabled_providers: self.enabled_providers,
            providers: self.providers.map(NativeProviderSettings::into_input),
            suggestion_indicators: self
                .suggestion_indicators
                .map(NativeSuggestionIndicators::into_input),
            show_vulnerabilities: self.show_vulnerabilities,
            show_suggestion_stats: self.show_suggestion_stats,
            show_prereleases: self.show_prereleases,
            http: self.http.map(NativeHttpConfig::into_input),
        })
    }
}

#[cfg(test)]
mod tests;
