mod response;
mod retry;

#[cfg(test)]
mod tests;

use crate::error::HttpError;
use crate::retry::RetryPolicy;

use response::{HttpResponse, read_response_text};
use retry::retry_or_fail;

pub(super) fn send_with_retries(
    method: &str,
    retry_policy: RetryPolicy,
    mut send: impl FnMut() -> Result<HttpResponse, ureq::Error>,
) -> Result<String, HttpError> {
    let mut attempt = 0;

    loop {
        if let Some(text) = send_attempt(&mut send, attempt, method, retry_policy)? {
            return Ok(text);
        }
        attempt += 1;
    }
}

fn send_attempt(
    send: &mut impl FnMut() -> Result<HttpResponse, ureq::Error>,
    attempt: u32,
    method: &str,
    retry_policy: RetryPolicy,
) -> Result<Option<String>, HttpError> {
    match send() {
        Ok(response) => read_response_text(response).map(Some),
        Err(error) => retry_or_fail(error, attempt, method, retry_policy),
    }
}
