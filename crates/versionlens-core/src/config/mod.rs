mod providers;
mod session;
mod settings;
mod suggestions;

pub use providers::{
    DependencyPropertyConfig, EnabledProviderConfig, FilePatternConfig, PrereleaseTagConfig,
    ProviderCacheConfig, ProviderHttpConfig, ProviderSettings, RegistryUrlConfig,
    dependency_property_config_from_name, dependency_property_manifest_kind_from_name,
    enabled_provider_config_from_name, file_pattern_config_from_name,
    file_pattern_manifest_kind_from_name, prerelease_tag_config_from_name,
    provider_cache_config_from_name, provider_http_config_from_name,
    provider_settings_manifest_kind_from_name, registry_url_config_from_name,
};
pub use session::{SessionConfig, UpdateSeverity};
pub use settings::Settings;
pub use suggestions::{SuggestionIndicators, standard_suggestion_indicators};

#[cfg(test)]
mod tests;
