use std::time::Duration;

const DEFAULT_CACHE_TTL_MS: u64 = 180_000;
const HALF_MILLI_NANOS: u128 = 500_000;
const MILLIS_PER_SECOND: u64 = 1000;
const NANOS_PER_MILLI: u128 = 1_000_000;
const SECONDS_PER_MINUTE: f64 = 60.0;

pub fn cache_ttl_ms(minutes: Option<f64>, seconds: Option<u32>) -> u64 {
    minutes.and_then(minutes_to_ms).unwrap_or_else(|| {
        seconds
            .map(|seconds| u64::from(seconds) * MILLIS_PER_SECOND)
            .unwrap_or(DEFAULT_CACHE_TTL_MS)
    })
}

pub fn minutes_to_ms(minutes: f64) -> Option<u64> {
    if !minutes.is_finite() || minutes < 0.0 {
        return None;
    }

    let seconds = minutes * SECONDS_PER_MINUTE;
    let duration = <Duration>::try_from_secs_f64(seconds).ok()?;
    let rounded_millis = duration
        .as_nanos()
        .checked_add(HALF_MILLI_NANOS)?
        .checked_div(NANOS_PER_MILLI)?;
    u64::try_from(rounded_millis).ok()
}

#[cfg(test)]
mod tests {
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
}
