use versionlens_cache::CacheKey;
use versionlens_model::Dependency;
use versionlens_providers::provider_id;

pub(crate) fn latest_cache_key(dependency: &Dependency) -> CacheKey {
    // Structured encoding keeps delimiter-bearing names and references distinct.
    let identity = serde_json::json!([
        dependency.name,
        dependency.requirement,
        dependency.hosted_url,
        dependency.hosted_name,
        dependency.group,
        dependency.canonical_reference,
    ]);
    let identity = if dependency.is_runtime_version() {
        serde_json::json!({
            "dependency": identity,
            "runtimeSource": versionlens_providers::RuntimeSource::for_dependency(dependency)
                .ok().map(|source| source.url()),
        })
    } else {
        identity
    };
    versionlens_cache::provider_package_cache_key(
        provider_id(dependency.ecosystem),
        &identity.to_string(),
    )
}

pub(crate) fn request_cache_key(url: &str, request_context_identity: u128) -> CacheKey {
    versionlens_cache::provider_package_cache_key(
        "request",
        &format!("{request_context_identity:032x}:{url}"),
    )
}

pub(crate) fn suggestion_cache_key(dependency: &Dependency, scope: &str) -> CacheKey {
    versionlens_cache::provider_package_cache_key(scope, latest_cache_key(dependency).as_str())
}

pub(crate) fn vulnerability_cache_key(dependency: &Dependency) -> CacheKey {
    versionlens_cache::provider_package_cache_key(
        "vulnerability",
        &serde_json::json!([
            dependency.ecosystem,
            dependency.name,
            dependency.requirement,
            dependency.hosted_name,
            dependency.hosted_url,
            dependency.group,
            dependency.canonical_reference,
        ])
        .to_string(),
    )
}
