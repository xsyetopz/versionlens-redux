use versionlens_cache::CacheKey;
use versionlens_model::{Dependency, Ecosystem};
use versionlens_providers::provider_id;

pub(crate) fn cache_key(ecosystem: Ecosystem, package: &str) -> CacheKey {
    versionlens_cache::provider_package_cache_key(provider_id(ecosystem), package)
}

pub(crate) fn latest_cache_key(dependency: &Dependency) -> CacheKey {
    versionlens_cache::provider_dependency_cache_key(
        provider_id(dependency.ecosystem),
        &dependency.name,
        &dependency.requirement,
    )
}

pub(crate) fn request_cache_key(url: &str, request_context_identity: u128) -> CacheKey {
    versionlens_cache::provider_package_cache_key(
        "request",
        &format!("{request_context_identity:032x}:{url}"),
    )
}

pub(crate) fn suggestion_cache_key(dependency: &Dependency) -> CacheKey {
    versionlens_cache::provider_dependency_cache_key(
        provider_id(dependency.ecosystem),
        &dependency.name,
        &dependency.requirement,
    )
}

pub(crate) fn vulnerability_cache_key(dependency: &Dependency) -> CacheKey {
    versionlens_cache::provider_dependency_cache_key(
        provider_id(dependency.ecosystem),
        &dependency.name,
        &dependency.requirement,
    )
}
