use super::{cache_ttl_ms, minutes_to_ms};

#[test]
fn legacy_fractional_minutes_convert_to_milliseconds() {
    assert_eq!(minutes_to_ms(0.5), Some(30_000));
    assert_eq!(cache_ttl_ms(Some(0.25), Some(90)), 15_000);
}

#[test]
fn invalid_minutes_fall_back_to_seconds_or_default_ttl() {
    assert_eq!(cache_ttl_ms(Some(f64::NAN), Some(30)), 30_000);
    assert_eq!(cache_ttl_ms(Some(-1.0), None), 180_000);
}
