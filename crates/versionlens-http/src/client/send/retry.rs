use std::thread::sleep;
use std::time::Duration;
use ureq::Error as UreqError;

use crate::error::HttpError;
use crate::retry::RetryPolicy;

use super::RequestDeadline;

pub(super) fn retry_or_fail(
    error: UreqError,
    attempt: u32,
    method: &str,
    policy: RetryPolicy,
    deadline: RequestDeadline,
) -> Result<Option<String>, HttpError> {
    if let Some(delay) = retry_delay(&error, attempt, method, policy) {
        if deadline
            .remaining()?
            .is_some_and(|remaining| remaining <= delay)
        {
            return Err(HttpError::DeadlineExceeded);
        }
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
