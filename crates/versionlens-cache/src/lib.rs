use std::time::{Duration as StdDuration, Instant as StdInstant};

mod entry;
mod key;
mod memory;
mod ttl;

pub use entry::CacheEntry;
pub use key::CacheKey;
pub use memory::MemoryCache;
pub use ttl::{cache_ttl_ms, minutes_to_ms};

pub(crate) fn default<T: Default>() -> T {
    <T as Default>::default()
}

pub(crate) fn now() -> StdInstant {
    std::time::Instant::now()
}

pub(crate) fn cache_entry<T>(value: T, ttl: StdDuration) -> CacheEntry<T> {
    entry::cache_entry(value, ttl)
}

#[cfg(test)]
pub(crate) fn memory_cache<T>(ttl: StdDuration) -> MemoryCache<T> {
    memory::memory_cache(ttl)
}

#[cfg(test)]
pub(crate) fn duration_from_mins(minutes: u64) -> StdDuration {
    std::time::Duration::from_mins(minutes)
}

pub fn provider_package_cache_key(provider: &str, package: &str) -> CacheKey {
    key::provider_package_cache_key(provider, package)
}

pub fn provider_dependency_cache_key(provider: &str, package: &str, requirement: &str) -> CacheKey {
    key::provider_dependency_cache_key(provider, package, requirement)
}
