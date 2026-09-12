use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::entry::CacheEntry;
use crate::key::CacheKey;

#[derive(Debug, Clone)]
struct Access {
    sequence: u64,
    weight: usize,
}

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
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: crate::default(),
            access: crate::default(),
            sequence: 0,
            capacity: 10_000,
            bytes: 0,
            byte_capacity: usize::MAX,
            heap_weight: |_| 0,
        }
    }

    pub fn insert(&mut self, key: CacheKey, value: T) {
        self.insert_with_ttl(key, value, self.ttl);
    }

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
        self.entries.insert(key, crate::cache_entry(value, ttl));
        self.evict();
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<&T> {
        self.get_with_expiry(key).map(|(value, _)| value)
    }

    pub fn get_with_expiry(&mut self, key: &CacheKey) -> Option<(&T, Instant)> {
        let now = crate::now();
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

    pub fn expires_at(&mut self, key: &CacheKey) -> Option<Instant> {
        self.get_with_expiry(key).map(|(_, deadline)| deadline)
    }

    pub fn purge_expired(&mut self) {
        self.prune();
    }

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
        let now = crate::now();
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

    pub fn clear(&mut self) {
        self.entries = HashMap::new();
        self.access = HashMap::new();
        self.bytes = 0;
    }
}

#[cfg(test)]
mod tests;
