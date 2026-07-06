use std::thread::sleep;
use std::time::Duration;
use ureq::Error as UreqError;

use crate::error::HttpError;
use crate::retry::RetryPolicy;

pub(super) fn retry_or_fail(
    error: UreqError,
    attempt: u32,
    method: &str,
    policy: RetryPolicy,
) -> Result<Option<String>, HttpError> {
    if let Some(delay) = retry_delay(&error, attempt, method, policy) {
        sleep(delay);
        Ok(None)
    } else {
        Err(error.into())
    }
}

fn retry_delay(
    error: &UreqError,
    attempt: u32,
    method: &str,
    policy: RetryPolicy,
) -> Option<Duration> {
    policy
        .retry_backoff_ms(attempt)
        .filter(|_| policy.should_retry_error(method, error))
        .map(crate::duration_from_millis)
}
