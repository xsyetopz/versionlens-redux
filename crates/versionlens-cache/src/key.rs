#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(String);

impl CacheKey {
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

pub fn provider_package_cache_key(provider: &str, package: &str) -> CacheKey {
    CacheKey(format!("{provider}:{package}"))
}

pub fn provider_dependency_cache_key(provider: &str, package: &str, requirement: &str) -> CacheKey {
    provider_package_cache_key(provider, &format!("{package}@{requirement}"))
}
