use std::time::{Duration, Instant};

use ureq::Error as UreqError;
mod response;
mod retry;

#[cfg(test)]
mod tests;

use crate::error::HttpError;
use crate::retry::RetryPolicy;

use response::{HttpResponse, read_response_text};
use retry::retry_or_fail;

#[derive(Clone, Copy)]
pub(super) struct RequestDeadline(Option<Instant>);

impl RequestDeadline {
    pub(super) fn after(timeout: Option<Duration>) -> Self {
        Self(timeout.and_then(|timeout| Instant::now().checked_add(timeout)))
    }

    fn remaining(self) -> Result<Option<Duration>, HttpError> {
        match self.0 {
            Some(deadline) => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    Err(HttpError::DeadlineExceeded)
                } else {
                    Ok(Some(remaining))
                }
            }
            None => Ok(None),
        }
    }

    fn ensure_remaining(self) -> Result<(), HttpError> {
        self.remaining().map(|_| ())
    }
}

pub(super) fn send_with_retries(
    method: &str,
    retry_policy: RetryPolicy,
    deadline: RequestDeadline,
    mut send: impl FnMut(Option<Duration>) -> Result<HttpResponse, UreqError>,
) -> Result<String, HttpError> {
    let mut attempt = 0;

    loop {
        if let Some(text) = send_attempt(&mut send, attempt, method, retry_policy, deadline)? {
            return Ok(text);
        }
        attempt += 1;
    }
}

fn send_attempt(
    send: &mut impl FnMut(Option<Duration>) -> Result<HttpResponse, UreqError>,
    attempt: u32,
    method: &str,
    retry_policy: RetryPolicy,
    deadline: RequestDeadline,
) -> Result<Option<String>, HttpError> {
    let remaining = deadline.remaining()?;
    match send(remaining) {
        Ok(response) => match read_response_text(response) {
            Ok(text) => {
                deadline.ensure_remaining()?;
                Ok(Some(text))
            }
            Err(_) if deadline.ensure_remaining().is_err() => Err(HttpError::DeadlineExceeded),
            Err(error) => Err(error),
        },
        Err(_) if deadline.ensure_remaining().is_err() => Err(HttpError::DeadlineExceeded),
        Err(error) => retry_or_fail(error, attempt, method, retry_policy, deadline),
    }
}
