use napi_derive::napi;

#[napi(object)]
pub struct NativeProviderSettings {
    pub registry_urls: Option<Vec<NativeRegistryUrl>>,
    pub prerelease_tag_filters: Option<Vec<NativePrereleaseTagFilter>>,
    pub provider_cache: Option<Vec<NativeProviderCacheConfig>>,
    pub provider_http: Option<Vec<NativeProviderHttpConfig>>,
    pub dependency_properties: Option<Vec<NativeDependencyPropertyConfig>>,
    pub file_patterns: Option<Vec<NativeFilePatternConfig>>,
}

#[napi(object)]
pub struct NativeDependencyPropertyConfig {
    pub ecosystem: String,
    pub provider: Option<String>,
    pub properties: Vec<String>,
}

#[napi(object)]
pub struct NativeFilePatternConfig {
    pub ecosystem: String,
    pub pattern: String,
}

#[napi(object)]
pub struct NativeRegistryUrl {
    pub ecosystem: String,
    pub url: String,
}

#[napi(object)]
pub struct NativePrereleaseTagFilter {
    pub ecosystem: String,
    pub tags: Vec<String>,
}

#[napi(object)]
pub struct NativeProviderHttpConfig {
    pub ecosystem: String,
    pub strict_ssl: Option<bool>,
}

#[napi(object)]
pub struct NativeProviderCacheConfig {
    pub ecosystem: String,
    pub cache_duration_minutes: Option<f64>,
}
