#[test]
fn returns_cached_value_before_ttl() {
    let mut cache = crate::memory_cache(crate::duration_from_mins(1));
    let key = crate::provider_package_cache_key("npm", "typescript");

    cache.insert(key.clone(), "6.0.3".to_owned());

    assert_eq!(key.as_str(), "npm:typescript");
    assert_eq!(cache.get(&key).map(|value| value.as_str()), Some("6.0.3"));
}

#[test]
fn expired_values_are_not_returned() {
    let mut cache = crate::memory_cache(crate::duration_from_mins(1));
    let key = crate::provider_package_cache_key("cargo", "serde");

    cache.insert_with_ttl(key.clone(), "1.0.228".to_owned(), std::time::Duration::ZERO);

    assert_eq!(cache.get(&key), None);
}

#[test]
fn clear_removes_cached_values() {
    let mut cache = crate::memory_cache(crate::duration_from_mins(1));
    let key = crate::provider_package_cache_key("cargo", "serde");

    cache.insert(key.clone(), "1.0.228".to_owned());
    cache.clear();

    assert_eq!(cache.get(&key), None);
}
