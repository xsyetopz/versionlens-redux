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
