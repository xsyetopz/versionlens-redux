use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::cache::entry::CacheEntry;
use crate::cache::key::CacheKey;

#[derive(Debug, Clone)]
struct Access {
    sequence: u64,
    weight: usize,
}

/// An in-memory cache with per-entry expiry and least-recently-used eviction by entry count and by estimated bytes.
#[derive(Debug, Clone)]
pub struct MemoryCache<T> {
    ttl: Duration,
    entries: HashMap<CacheKey, CacheEntry<T>>,
    access: HashMap<CacheKey, Access>,
    sequence: u64,
    capacity: usize,
    bytes: usize,
    byte_capacity: usize,
    heap_weight: fn(&T) -> usize,
}

impl<T> MemoryCache<T> {
    /// An empty cache whose entries live for `ttl` unless inserted with their own.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: HashMap::default(),
            access: HashMap::default(),
            sequence: 0,
            capacity: 10_000,
            bytes: 0,
            byte_capacity: usize::MAX,
            heap_weight: |_| 0,
        }
    }

    /// Stores `value` under `key` with the cache default time-to-live.
    pub fn insert(&mut self, key: CacheKey, value: T) {
        self.insert_with_ttl(key, value, self.ttl);
    }

    /// Stores `value` under `key` for `ttl`; a value heavier than the byte capacity is not stored.
    pub fn insert_with_ttl(&mut self, key: CacheKey, value: T, ttl: Duration) {
        self.remove(&key);
        let weight = self.weight(&key, &value);
        if weight > self.byte_capacity {
            return;
        }
        if self.sequence.is_multiple_of(64) || self.entries.len() >= self.capacity {
            self.prune();
        }
        self.sequence = self.sequence.saturating_add(1);
        self.bytes = self.bytes.saturating_add(weight);
        self.access.insert(
            key.clone(),
            Access {
                sequence: self.sequence,
                weight,
            },
        );
        self.entries.insert(key, CacheEntry::new(value, ttl));
        self.evict();
    }

    /// The unexpired value for `key`, marking it as recently used.
    pub fn get(&mut self, key: &CacheKey) -> Option<&T> {
        self.get_with_expiry(key).map(|(value, _)| value)
    }

    /// The unexpired value for `key` with its expiry instant, marking it as recently used.
    pub fn get_with_expiry(&mut self, key: &CacheKey) -> Option<(&T, Instant)> {
        let now = Instant::now();
        if self
            .entries
            .get(key)
            .is_some_and(|entry| entry.is_expired_at(now))
        {
            self.remove(key);
            return None;
        }
        if let Some(access) = self.access.get_mut(key) {
            self.sequence = self.sequence.saturating_add(1);
            access.sequence = self.sequence;
        }
        self.entries
            .get(key)
            .map(|entry| (entry.value(), entry.expires_at()))
    }

    /// The expiry instant of the unexpired entry for `key`.
    pub fn expires_at(&mut self, key: &CacheKey) -> Option<Instant> {
        self.get_with_expiry(key).map(|(_, deadline)| deadline)
    }

    /// Drops every expired entry.
    pub fn purge_expired(&mut self) {
        self.prune();
    }

    /// Limits the cache to `capacity` entries, evicting the least recently used beyond that.
    #[must_use]
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self.evict();
        self
    }

    /// The callback measures heap allocations owned by the value.
    #[must_use]
    pub fn with_byte_capacity(mut self, capacity: usize, heap_weight: fn(&T) -> usize) -> Self {
        self.byte_capacity = capacity;
        self.heap_weight = heap_weight;
        for (key, entry) in &self.entries {
            let weight = self.weight(key, entry.value());
            if let Some(access) = self.access.get_mut(key) {
                access.weight = weight;
            }
        }
        self.recount();
        self.evict();
        self
    }

    fn weight(&self, key: &CacheKey, value: &T) -> usize {
        // Reserve table capacity and control-byte overhead in both hash maps.
        let table =
            4 * (2 * size_of::<CacheKey>() + size_of::<CacheEntry<T>>() + size_of::<Access>() + 16);
        (self.heap_weight)(value)
            .saturating_add(2 * key.heap_bytes())
            .saturating_add(table)
    }

    fn remove(&mut self, key: &CacheKey) {
        self.entries.remove(key);
        if let Some(access) = self.access.remove(key) {
            self.bytes = self.bytes.saturating_sub(access.weight);
        }
    }

    fn evict(&mut self) {
        while self.entries.len() > self.capacity || self.bytes > self.byte_capacity {
            let victim = self
                .access
                .iter()
                .min_by_key(|(_, access)| access.sequence)
                .map(|(key, _)| key.clone());
            let Some(victim) = victim else {
                break;
            };
            self.remove(&victim);
        }
        self.shrink();
    }

    fn recount(&mut self) {
        self.bytes = self
            .access
            .values()
            .fold(0_usize, |bytes, access| bytes.saturating_add(access.weight));
    }

    fn prune(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| !entry.is_expired_at(now));
        self.access.retain(|key, _| self.entries.contains_key(key));
        self.recount();
        self.shrink();
    }

    fn shrink(&mut self) {
        if self.entries.len() < self.entries.capacity() / 4 {
            self.entries.shrink_to_fit();
            self.access.shrink_to_fit();
        }
    }

    /// Drops every entry.
    pub fn clear(&mut self) {
        self.entries = HashMap::new();
        self.access = HashMap::new();
        self.bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn returns_cached_value_before_ttl() {
        let mut cache = crate::cache::MemoryCache::new(Duration::from_secs(60));
        let key = crate::cache::provider_package_cache_key("npm", "typescript");

        cache.insert(key.clone(), "6.0.3".to_owned());

        assert_eq!(key.as_str(), "npm:typescript");
        assert_eq!(cache.get(&key).map(String::as_str), Some("6.0.3"));
    }

    #[test]
    fn expired_values_are_not_returned() {
        let mut cache = crate::cache::MemoryCache::new(Duration::from_secs(60));
        let key = crate::cache::provider_package_cache_key("cargo", "serde");

        cache.insert_with_ttl(key.clone(), "1.0.228".to_owned(), Duration::ZERO);

        assert_eq!(cache.get(&key), None);
        assert!(!cache.entries.contains_key(&key));
    }

    #[test]
    fn clear_removes_cached_values() {
        let mut cache = crate::cache::MemoryCache::new(Duration::from_secs(60));
        let key = crate::cache::provider_package_cache_key("cargo", "serde");

        cache.insert(key.clone(), "1.0.228".to_owned());
        cache.clear();

        assert_eq!(cache.get(&key), None);
    }
    use std::time::Duration;

    #[test]
    fn capacity_retains_recently_accessed_values() {
        let mut cache = crate::cache::MemoryCache::new(Duration::from_secs(60)).with_capacity(2);
        let keys = ["first", "second", "third"]
            .map(|name| crate::cache::provider_package_cache_key("npm", name));
        cache.insert(keys[0].clone(), 1);
        cache.insert(keys[1].clone(), 2);
        assert_eq!(cache.get(&keys[0]), Some(&1));
        cache.insert(keys[2].clone(), 3);
        assert_eq!(cache.get(&keys[0]), Some(&1));
        assert_eq!(cache.get(&keys[1]), None);
        assert_eq!(cache.get(&keys[2]), Some(&3));
    }

    #[test]
    fn byte_budget_preserves_recent_values_and_releases_allocations_on_clear() {
        let mut cache = crate::cache::MemoryCache::new(Duration::from_secs(60))
            .with_byte_capacity(1600, String::capacity);
        let keys = ["first", "second", "third"]
            .map(|name| crate::cache::provider_package_cache_key("npm", name));
        cache.insert(keys[0].clone(), "a".repeat(200));
        cache.insert(keys[1].clone(), "b".repeat(200));
        assert!(cache.get(&keys[0]).is_some());
        cache.insert(keys[2].clone(), "c".repeat(200));
        assert!(cache.get(&keys[0]).is_some());
        assert!(cache.get(&keys[1]).is_none());
        assert!(cache.get(&keys[2]).is_some());
        assert!(cache.bytes <= 1600);
        cache.clear();
        assert_eq!(cache.entries.capacity(), 0);
        assert_eq!(cache.access.capacity(), 0);
        assert_eq!(cache.bytes, 0);
    }

    #[test]
    fn a_value_larger_than_the_budget_does_not_displace_other_values() {
        let mut cache = crate::cache::MemoryCache::new(Duration::from_secs(60))
            .with_byte_capacity(1600, String::capacity);
        let first = crate::cache::provider_package_cache_key("npm", "first");
        let large = crate::cache::provider_package_cache_key("npm", "large");
        cache.insert(first.clone(), "a".repeat(100));
        cache.insert(large.clone(), "b".repeat(2000));
        assert!(cache.get(&first).is_some());
        assert!(cache.get(&large).is_none());
        assert!(cache.bytes <= 1600);
    }

    #[test]
    fn expiry_deadlines_follow_replacement_and_clear() {
        let mut cache = crate::cache::MemoryCache::new(Duration::from_secs(60));
        let key = crate::cache::provider_package_cache_key("npm", "example");
        assert_eq!(cache.expires_at(&key), None);
        cache.insert(key.clone(), 1);
        let first = cache.expires_at(&key).expect("cached deadline");
        cache.insert_with_ttl(key.clone(), 2, Duration::from_secs(120));
        assert!(cache.expires_at(&key).expect("replacement deadline") > first);
        cache.clear();
        assert_eq!(cache.expires_at(&key), None);
    }

    #[test]
    fn expired_entries_release_memory_without_individual_reads() {
        let mut cache = crate::cache::MemoryCache::new(Duration::ZERO);
        for name in ["first", "second"] {
            cache.insert(
                crate::cache::provider_package_cache_key("npm", name),
                name.to_owned(),
            );
        }
        cache.purge_expired();
        assert!(cache.entries.is_empty());
        assert!(cache.access.is_empty());
        assert_eq!(cache.bytes, 0);
    }
}
