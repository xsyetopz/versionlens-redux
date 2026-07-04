use versionlens_cache::CacheKey;
use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_providers::provider_id;

pub(crate) fn cache_key(ecosystem: Ecosystem, package: &str) -> CacheKey {
    CacheKey::provider_package(provider_id(ecosystem), package)
}

pub(crate) fn latest_cache_key(dependency: &Dependency) -> CacheKey {
    CacheKey::provider_dependency(
        provider_id(dependency.ecosystem),
        &dependency.name,
        &dependency.requirement,
    )
}

pub(crate) fn request_cache_key(url: &str) -> CacheKey {
    CacheKey::provider_package("request", url)
}

pub(crate) fn suggestion_cache_key(dependency: &Dependency) -> CacheKey {
    CacheKey::provider_dependency(
        provider_id(dependency.ecosystem),
        &dependency.name,
        &dependency.requirement,
    )
}

pub(crate) fn vulnerability_cache_key(dependency: &Dependency) -> CacheKey {
    CacheKey::provider_dependency(
        provider_id(dependency.ecosystem),
        &dependency.name,
        &dependency.requirement,
    )
}
