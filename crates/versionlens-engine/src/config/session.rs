use crate::Severity;
use serde::{Deserialize, Serialize};
use versionlens_fetch::cache_ttl_ms;
use versionlens_fetch::{HttpConfig, standard_http_config};

use super::{
    EnabledProviderConfig, ProviderSettings, SuggestionIndicators, standard_suggestion_indicators,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The engine settings of one session.
pub struct SessionConfig {
    /// How long a cached registry answer stays fresh, in milliseconds.
    pub cache_ttl_ms: u64,
    /// The providers that are on. Empty means all of them.
    pub enabled_providers: Vec<EnabledProviderConfig>,
    /// Provider settings: file patterns, registry URLs, prerelease tags, caching, and HTTP.
    pub providers: ProviderSettings,
    /// The markers shown in suggestion titles.
    pub suggestion_indicators: SuggestionIndicators,
    /// Whether vulnerability checks run.
    pub show_vulnerabilities: bool,
    /// Whether suggestion titles include statistics.
    pub show_suggestion_stats: bool,
    /// Whether prerelease versions are offered.
    pub show_prereleases: bool,
    /// The severity of `update-available` diagnostics, or `None` for none.
    pub update_severity: Option<Severity>,
    /// The HTTP settings for registry requests.
    pub http: HttpConfig,
}

/// The `versionlens.diagnostics.updateSeverity` setting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateSeverity {
    /// Report as an error.
    Error,
    /// Report as a warning.
    Warn,
    /// The default, because VS Code's Problems panel does not list hints.
    #[default]
    Info,
    /// Report as a hint.
    Hint,
    /// Report nothing.
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
