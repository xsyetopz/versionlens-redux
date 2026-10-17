#[test]
fn provider_dependency_cache_key_includes_requirement() {
    let key = crate::cache::provider_dependency_cache_key("npm", "typescript", "^6.0.3");

    assert_eq!(key.as_str(), "npm:typescript@^6.0.3");
}
