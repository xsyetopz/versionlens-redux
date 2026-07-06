use versionlens_core::ProviderSettingsInput;

use super::convert::{
    dependency_property_input_from_native, file_pattern_input_from_native,
    prerelease_tag_input_from_native, provider_cache_input_from_native,
    provider_http_input_from_native, registry_url_input_from_native,
};
use super::types::NativeProviderSettings;

impl NativeProviderSettings {
    pub(in crate::model::config) fn into_input(self) -> ProviderSettingsInput {
        ProviderSettingsInput {
            registry_urls: self.registry_urls.map(|urls| {
                urls.into_iter()
                    .map(registry_url_input_from_native)
                    .collect()
            }),
            prerelease_tag_filters: self.prerelease_tag_filters.map(|filters| {
                filters
                    .into_iter()
                    .map(prerelease_tag_input_from_native)
                    .collect()
            }),
            provider_cache: self.provider_cache.map(|configs| {
                configs
                    .into_iter()
                    .map(provider_cache_input_from_native)
                    .collect()
            }),
            provider_http: self.provider_http.map(|configs| {
                configs
                    .into_iter()
                    .map(provider_http_input_from_native)
                    .collect()
            }),
            dependency_properties: self.dependency_properties.map(|configs| {
                configs
                    .into_iter()
                    .map(dependency_property_input_from_native)
                    .collect()
            }),
            file_patterns: self.file_patterns.map(|configs| {
                configs
                    .into_iter()
                    .map(file_pattern_input_from_native)
                    .collect()
            }),
        }
    }
}

impl From<NativeProviderSettings> for ProviderSettingsInput {
    fn from(value: NativeProviderSettings) -> Self {
        value.into_input()
    }
}
