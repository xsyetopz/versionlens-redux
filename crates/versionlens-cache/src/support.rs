use std::time::{Duration as StdDuration, Instant as StdInstant};

use crate::{CacheEntry, entry};

pub(crate) fn default<T: Default>() -> T {
    <T as Default>::default()
}

pub(crate) fn now() -> StdInstant {
    StdInstant::now()
}

pub(crate) fn cache_entry<T>(value: T, ttl: StdDuration) -> CacheEntry<T> {
    entry::cache_entry(value, ttl)
}
