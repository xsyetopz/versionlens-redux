#[test]
fn returns_cached_value_before_ttl() {
    let mut cache = crate::MemoryCache::new(Duration::from_secs(60));
    let key = crate::provider_package_cache_key("npm", "typescript");

    cache.insert(key.clone(), "6.0.3".to_owned());

    assert_eq!(key.as_str(), "npm:typescript");
    assert_eq!(cache.get(&key).map(|value| value.as_str()), Some("6.0.3"));
}

#[test]
fn expired_values_are_not_returned() {
    let mut cache = crate::MemoryCache::new(Duration::from_secs(60));
    let key = crate::provider_package_cache_key("cargo", "serde");

    cache.insert_with_ttl(key.clone(), "1.0.228".to_owned(), std::time::Duration::ZERO);

    assert_eq!(cache.get(&key), None);
    assert!(!cache.entries.contains_key(&key));
}

#[test]
fn clear_removes_cached_values() {
    let mut cache = crate::MemoryCache::new(Duration::from_secs(60));
    let key = crate::provider_package_cache_key("cargo", "serde");

    cache.insert(key.clone(), "1.0.228".to_owned());
    cache.clear();

    assert_eq!(cache.get(&key), None);
}
use std::time::Duration;

#[test]
fn capacity_retains_recently_accessed_values() {
    let mut cache = crate::MemoryCache::new(Duration::from_secs(60)).with_capacity(2);
    let keys =
        ["first", "second", "third"].map(|name| crate::provider_package_cache_key("npm", name));
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
    let mut cache =
        crate::MemoryCache::new(Duration::from_secs(60)).with_byte_capacity(1600, String::capacity);
    let keys =
        ["first", "second", "third"].map(|name| crate::provider_package_cache_key("npm", name));
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
    let mut cache =
        crate::MemoryCache::new(Duration::from_secs(60)).with_byte_capacity(1600, String::capacity);
    let first = crate::provider_package_cache_key("npm", "first");
    let large = crate::provider_package_cache_key("npm", "large");
    cache.insert(first.clone(), "a".repeat(100));
    cache.insert(large.clone(), "b".repeat(2000));
    assert!(cache.get(&first).is_some());
    assert!(cache.get(&large).is_none());
    assert!(cache.bytes <= 1600);
}

#[test]
fn expiry_deadlines_follow_replacement_and_clear() {
    let mut cache = crate::MemoryCache::new(Duration::from_secs(60));
    let key = crate::provider_package_cache_key("npm", "example");
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
    let mut cache = crate::MemoryCache::new(Duration::ZERO);
    for name in ["first", "second"] {
        cache.insert(
            crate::provider_package_cache_key("npm", name),
            name.to_owned(),
        );
    }
    cache.purge_expired();
    assert!(cache.entries.is_empty());
    assert!(cache.access.is_empty());
    assert_eq!(cache.bytes, 0);
}
