use std::future::Future;
use std::time::{Duration, Instant};

use reqwest::{RequestBuilder, Response};

use crate::client::HttpBytesResult;
use crate::config::HttpConfig;
use crate::error::HttpError;
use crate::retry::RetryPolicy;

pub(super) const MAX_RESPONSE_BODY_BYTES: usize = 64 * 1024 * 1024;

pub(super) async fn send(
    request: RequestBuilder,
    config: &HttpConfig,
    deadline: RequestDeadline,
) -> Result<Response, reqwest::Error> {
    let request = match deadline.remaining() {
        Ok(Some(remaining)) => {
            request.timeout(Duration::from_millis(config.timeout_ms).min(remaining))
        }
        _ => request,
    };
    request.send().await?.error_for_status()
}

pub(super) async fn send_with_retries<F, Fut>(
    method: &str,
    retry_policy: RetryPolicy,
    deadline: RequestDeadline,
    mut send: F,
) -> HttpBytesResult
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Response, reqwest::Error>>,
{
    let mut attempt = 0;
    loop {
        deadline.ensure_remaining()?;
        match send().await {
            Ok(response) => {
                let result = read_response_bytes(response).await;
                if deadline.ensure_remaining().is_err() {
                    return Err(HttpError::DeadlineExceeded);
                }
                return result;
            }
            Err(error) => {
                if deadline.ensure_remaining().is_err() {
                    return Err(HttpError::DeadlineExceeded);
                }
                let Some(delay) = retry_policy
                    .retry_backoff_ms(attempt)
                    .filter(|_| retry_policy.should_retry_error(method, &error))
                    .map(Duration::from_millis)
                else {
                    return Err(error.into());
                };
                if deadline
                    .remaining()?
                    .is_some_and(|remaining| remaining <= delay)
                {
                    return Err(HttpError::DeadlineExceeded);
                }
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}

async fn read_response_bytes(mut response: Response) -> HttpBytesResult {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BODY_BYTES as u64)
    {
        return Err(HttpError::ResponseTooLarge);
    }

    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BODY_BYTES {
            return Err(HttpError::ResponseTooLarge);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

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

#[cfg(test)]
mod tests;
