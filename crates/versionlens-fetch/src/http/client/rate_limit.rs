use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderMap, RETRY_AFTER};
use reqwest::{Response, Url};

use crate::http::error::HttpError;

/// Host and credential pairs whose rate limit is exhausted, with the status
/// that reported it and the time it resets. Process-wide, because every
/// session that sends the same credential shares its quota.
static HELD_HOSTS: OnceLock<Mutex<HashMap<(String, u64), Hold>>> = OnceLock::new();

#[derive(Clone, Copy)]
struct Hold {
    status: u16,
    reset: SystemTime,
}

/// Fails without a request while the host of `url` is held for `credential`.
pub(super) fn ensure_open(url: &str, credential: u64) -> Result<(), HttpError> {
    let Some(host) = host_key(url) else {
        return Ok(());
    };
    let key = (host, credential);
    let mut table = held_hosts();
    let active = match table.get(&key).copied() {
        Some(hold) if SystemTime::now() < hold.reset => Some(hold),
        Some(_) => {
            table.remove(&key);
            None
        }
        None => None,
    };
    drop(table);
    match active {
        Some(hold) => Err(HttpError::RateLimited {
            host: key.0,
            status: hold.status,
            reset: hold.reset,
        }),
        None => Ok(()),
    }
}

/// Holds the host of `url` for `credential` when `response` reports an
/// exhausted rate limit, and returns the error for it.
pub(super) fn hold_if_limited(
    url: &str,
    credential: u64,
    response: &Response,
) -> Option<HttpError> {
    let status = response.status().as_u16();
    if !matches!(status, 403 | 429) {
        return None;
    }
    let reset = reset_time(response.headers(), SystemTime::now())?;
    let host = host_key(url)?;
    held_hosts().insert((host.clone(), credential), Hold { status, reset });
    Some(HttpError::RateLimited {
        host,
        status,
        reset,
    })
}

/// Reads the reset time from `retry-after` (seconds) or from an exhausted
/// `x-ratelimit-remaining` with `x-ratelimit-reset` (epoch seconds).
fn reset_time(headers: &HeaderMap, now: SystemTime) -> Option<SystemTime> {
    let number = |name| headers.get(name)?.to_str().ok()?.trim().parse::<u64>().ok();
    if let Some(seconds) = number(RETRY_AFTER.as_str()) {
        return now.checked_add(Duration::from_secs(seconds));
    }
    if number("x-ratelimit-remaining")? != 0 {
        return None;
    }
    UNIX_EPOCH.checked_add(Duration::from_secs(number("x-ratelimit-reset")?))
}

fn host_key(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    Some(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host,
    })
}

fn held_hosts() -> std::sync::MutexGuard<'static, HashMap<(String, u64), Hold>> {
    HELD_HOSTS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
