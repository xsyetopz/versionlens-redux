use std::time::{Duration, Instant};

/// A value that stops being returned once its time-to-live has elapsed.
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    /// Wraps `value` so that it expires `ttl` from now.
    pub fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    /// The value, or `None` once the entry has expired.
    pub fn get(&self) -> Option<&T> {
        (Instant::now() < self.expires_at).then_some(&self.value)
    }

    pub(crate) fn is_expired_at(&self, now: Instant) -> bool {
        now >= self.expires_at
    }

    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    pub(crate) fn expires_at(&self) -> Instant {
        self.expires_at
    }
}
