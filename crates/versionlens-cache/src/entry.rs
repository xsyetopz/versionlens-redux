use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: crate::now() + ttl,
        }
    }

    pub fn get(&self) -> Option<&T> {
        (crate::now() < self.expires_at).then_some(&self.value)
    }

    pub(crate) fn is_expired_at(&self, now: Instant) -> bool {
        now >= self.expires_at
    }

    pub(crate) fn value(&self) -> &T {
        &self.value
    }
}

pub(crate) fn cache_entry<T>(value: T, ttl: Duration) -> CacheEntry<T> {
    CacheEntry {
        value,
        expires_at: crate::now() + ttl,
    }
}
