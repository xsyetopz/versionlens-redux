/// An opaque cache lookup key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(String);

impl CacheKey {
    /// A key that is the lowercase hex SHA-256 digest of `bytes`.
    pub fn content(bytes: &[u8]) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let digest = ring::digest::digest(&ring::digest::SHA256, bytes);
        let mut encoded = String::with_capacity(digest.as_ref().len() * 2);
        for byte in digest.as_ref() {
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 15)]));
        }
        Self(encoded)
    }

    pub(crate) fn heap_bytes(&self) -> usize {
        self.0.capacity()
    }

    /// A key for `package` as served by `provider`.
    pub fn provider_package(provider: &str, package: &str) -> Self {
        Self(format!("{provider}:{package}"))
    }

    /// A key for the `name` dependency with version `requirement` as served by `provider`.
    pub fn provider_dependency(provider: &str, name: &str, requirement: &str) -> Self {
        Self::provider_package(provider, &format!("{name}@{requirement}"))
    }

    /// The key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A key for `package` as served by `provider`.
pub fn provider_package_cache_key(provider: &str, package: &str) -> CacheKey {
    CacheKey(format!("{provider}:{package}"))
}

/// A key for `package` constrained by `requirement` as served by `provider`.
pub fn provider_dependency_cache_key(provider: &str, package: &str, requirement: &str) -> CacheKey {
    provider_package_cache_key(provider, &format!("{package}@{requirement}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn provider_dependency_cache_key_includes_requirement() {
        let key = crate::cache::provider_dependency_cache_key("npm", "typescript", "^6.0.3");

        assert_eq!(key.as_str(), "npm:typescript@^6.0.3");
    }
}
