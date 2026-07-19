use versionlens_core::{
    DependencyPropertyConfigInput, FilePatternConfigInput, PrereleaseTagConfigInput,
    ProviderCacheConfigInput, ProviderHttpConfigInput, RegistryUrlConfigInput,
};

use super::objects::{
    NativeDependencyPropertyConfig, NativeFilePatternConfig, NativePrereleaseTagFilter,
    NativeProviderCacheConfig, NativeProviderHttpConfig, NativeRegistryUrl,
};

pub(super) fn dependency_property_input_from_native(
    config: NativeDependencyPropertyConfig,
) -> DependencyPropertyConfigInput {
    DependencyPropertyConfigInput {
        ecosystem: config.ecosystem,
        provider: config.provider,
        properties: config.properties,
    }
}

pub(super) fn file_pattern_input_from_native(
    config: NativeFilePatternConfig,
) -> FilePatternConfigInput {
    FilePatternConfigInput {
        ecosystem: config.ecosystem,
        pattern: config.pattern,
    }
}

pub(super) fn registry_url_input_from_native(url: NativeRegistryUrl) -> RegistryUrlConfigInput {
    RegistryUrlConfigInput {
        ecosystem: url.ecosystem,
        url: url.url,
    }
}

pub(super) fn prerelease_tag_input_from_native(
    filter: NativePrereleaseTagFilter,
) -> PrereleaseTagConfigInput {
    PrereleaseTagConfigInput {
        ecosystem: filter.ecosystem,
        tags: filter.tags,
    }
}

pub(super) fn provider_cache_input_from_native(
    config: NativeProviderCacheConfig,
) -> ProviderCacheConfigInput {
    ProviderCacheConfigInput {
        ecosystem: config.ecosystem,
        cache_duration_minutes: config.cache_duration_minutes,
    }
}

pub(super) fn provider_http_input_from_native(
    config: NativeProviderHttpConfig,
) -> ProviderHttpConfigInput {
    ProviderHttpConfigInput {
        ecosystem: config.ecosystem,
        strict_ssl: config.strict_ssl,
    }
}
