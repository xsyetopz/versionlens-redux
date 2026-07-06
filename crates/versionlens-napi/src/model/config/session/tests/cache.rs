use super::blank_session_config;
use crate::model::config::NativeSessionConfig;

#[test]
fn missing_cache_ttl_uses_extension_default() {
    let config = blank_session_config().into_core();

    assert_eq!(config.cache_ttl_ms, 180_000);
    assert_eq!(config.http.timeout_ms, 10_000);
    assert!(config.http.strict_ssl);
}

#[test]
fn legacy_fractional_cache_minutes_are_converted_to_milliseconds() {
    let config = NativeSessionConfig {
        cache_duration_minutes: Some(0.5),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(config.cache_ttl_ms, 30_000);
}
