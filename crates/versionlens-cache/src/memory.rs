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
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: CacheKey, value: T) {
        self.entries.insert(key, CacheEntry::new(value, self.ttl));
    }

    pub fn insert_with_ttl(&mut self, key: CacheKey, value: T, ttl: Duration) {
        self.entries.insert(key, CacheEntry::new(value, ttl));
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
