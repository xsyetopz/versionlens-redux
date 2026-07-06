mod providers;
mod session;
mod suggestions;

pub use providers::{
    DependencyPropertyConfig, DependencyPropertyConfigInput, EnabledProviderConfig,
    FilePatternConfig, FilePatternConfigInput, PrereleaseTagConfig, PrereleaseTagConfigInput,
    ProviderCacheConfig, ProviderCacheConfigInput, ProviderHttpConfig, ProviderHttpConfigInput,
    ProviderSettings, ProviderSettingsInput, RegistryUrlConfig, RegistryUrlConfigInput,
    dependency_property_config_from_name, dependency_property_manifest_kind_from_name,
    enabled_provider_config_from_name, file_pattern_config_from_name,
    file_pattern_manifest_kind_from_name, prerelease_tag_config_from_name,
    provider_cache_config_from_input, provider_cache_config_from_name,
    provider_http_config_from_name, provider_settings_manifest_kind_from_name,
    registry_url_config_from_name,
};
pub use session::{SessionConfig, SessionConfigInput};
pub use suggestions::{
    SuggestionIndicators, SuggestionIndicatorsInput, standard_suggestion_indicators,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) fn session_config_from_input(input: SessionConfigInput) -> SessionConfig {
    input.into()
}
