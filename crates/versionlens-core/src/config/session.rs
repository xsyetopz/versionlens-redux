use crate::Severity;
use serde::{Deserialize, Serialize};
use versionlens_fetch::cache_ttl_ms;
use versionlens_fetch::{HttpConfig, standard_http_config};

use super::{
    EnabledProviderConfig, ProviderSettings, SuggestionIndicators, standard_suggestion_indicators,
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
    /// The severity of `update-available` diagnostics, or `None` for none.
    pub update_severity: Option<Severity>,
    pub http: HttpConfig,
}

/// The `versionlens.diagnostics.updateSeverity` setting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateSeverity {
    Error,
    Warn,
    /// The default, because VS Code's Problems panel does not list hints.
    #[default]
    Info,
    Hint,
    Off,
}

impl UpdateSeverity {
    pub(super) const fn severity(self) -> Option<Severity> {
        match self {
            Self::Error => Some(Severity::Error),
            Self::Warn => Some(Severity::Warning),
            Self::Info => Some(Severity::Information),
            Self::Hint => Some(Severity::Hint),
            Self::Off => None,
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cache_ttl_ms: cache_ttl_ms(None, None),
            enabled_providers: Vec::new(),
            providers: ProviderSettings::default(),
            suggestion_indicators: standard_suggestion_indicators(),
            show_vulnerabilities: true,
            show_suggestion_stats: false,
            show_prereleases: false,
            update_severity: UpdateSeverity::default().severity(),
            http: standard_http_config(),
        }
    }
}
