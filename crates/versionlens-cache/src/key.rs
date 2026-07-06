#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(String);

impl CacheKey {
    pub fn provider_package(provider: &str, package: &str) -> Self {
        Self(format!("{provider}:{package}"))
    }

    pub fn provider_dependency(provider: &str, name: &str, requirement: &str) -> Self {
        Self::provider_package(provider, &format!("{name}@{requirement}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests;

pub(crate) fn provider_package_cache_key(provider: &str, package: &str) -> CacheKey {
    CacheKey(format!("{provider}:{package}"))
}

pub(crate) fn provider_dependency_cache_key(
    provider: &str,
    package: &str,
    requirement: &str,
) -> CacheKey {
    provider_package_cache_key(provider, &format!("{package}@{requirement}"))
}
