use std::collections::HashMap;
use std::time::Duration;

use crate::entry::CacheEntry;
use crate::key::CacheKey;

#[derive(Debug, Clone)]
pub struct MemoryCache<T> {
    ttl: Duration,
    entries: HashMap<CacheKey, CacheEntry<T>>,
}

impl<T> MemoryCache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: crate::default(),
        }
    }

    pub fn insert(&mut self, key: CacheKey, value: T) {
        self.entries
            .insert(key, crate::cache_entry(value, self.ttl));
    }

    pub fn insert_with_ttl(&mut self, key: CacheKey, value: T, ttl: Duration) {
        self.entries.insert(key, crate::cache_entry(value, ttl));
    }

    pub fn get(&self, key: &CacheKey) -> Option<&T> {
        self.entries.get(key)?.get()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) fn memory_cache<T>(ttl: Duration) -> MemoryCache<T> {
    MemoryCache {
        ttl,
        entries: crate::default(),
    }
}
