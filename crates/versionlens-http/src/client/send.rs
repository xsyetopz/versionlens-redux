use std::time::{Duration, Instant};

use ureq::Error as UreqError;
mod response;
mod retry;

#[cfg(test)]
mod tests;

use crate::error::HttpError;
use crate::retry::RetryPolicy;

pub(super) use response::{HttpResponse, read_response_bytes, read_response_text};
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

pub(super) fn send_with_retries<T>(
    method: &str,
    retry_policy: RetryPolicy,
    deadline: RequestDeadline,
    mut read_response: impl FnMut(HttpResponse) -> Result<T, HttpError>,
    mut send: impl FnMut(Option<Duration>) -> Result<HttpResponse, UreqError>,
) -> Result<T, HttpError> {
    let mut attempt = 0;

    loop {
        if let Some(response) = send_attempt(
            &mut send,
            &mut read_response,
            attempt,
            method,
            retry_policy,
            deadline,
        )? {
            return Ok(response);
        }
        attempt += 1;
    }
}

fn send_attempt<T>(
    send: &mut impl FnMut(Option<Duration>) -> Result<HttpResponse, UreqError>,
    read_response: &mut impl FnMut(HttpResponse) -> Result<T, HttpError>,
    attempt: u32,
    method: &str,
    retry_policy: RetryPolicy,
    deadline: RequestDeadline,
) -> Result<Option<T>, HttpError> {
    let remaining = deadline.remaining()?;
    match send(remaining) {
        Ok(response) => match read_response(response) {
            Ok(response) => {
                deadline.ensure_remaining()?;
                Ok(Some(response))
            }
            Err(_) if deadline.ensure_remaining().is_err() => Err(HttpError::DeadlineExceeded),
            Err(error) => Err(error),
        },
        Err(_) if deadline.ensure_remaining().is_err() => Err(HttpError::DeadlineExceeded),
        Err(error) => {
            retry_or_fail(error, attempt, method, retry_policy, deadline)?;
            Ok(None)
        }
    }
}
