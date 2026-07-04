use std::time::Duration;

use crate::key::CacheKey;

use super::MemoryCache;

#[test]
fn returns_cached_value_before_ttl() {
    let mut cache = MemoryCache::new(Duration::from_mins(1));
    let key = CacheKey::provider_package("npm", "typescript");

    cache.insert(key.clone(), "6.0.3".to_owned());

    assert_eq!(key.as_str(), "npm:typescript");
    assert_eq!(cache.get(&key).map(String::as_str), Some("6.0.3"));
}

#[test]
fn expired_values_are_not_returned() {
    let mut cache = MemoryCache::new(Duration::from_mins(1));
    let key = CacheKey::provider_package("cargo", "serde");

    cache.insert_with_ttl(key.clone(), "1.0.228".to_owned(), Duration::ZERO);

    assert_eq!(cache.get(&key), None);
}

#[test]
fn clear_removes_cached_values() {
    let mut cache = MemoryCache::new(Duration::from_mins(1));
    let key = CacheKey::provider_package("cargo", "serde");

    cache.insert(key.clone(), "1.0.228".to_owned());
    cache.clear();

    assert_eq!(cache.get(&key), None);
}
