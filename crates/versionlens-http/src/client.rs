mod agent;
mod headers;
mod send;

use std::time::Duration;

use crate::config::HttpConfig;
use crate::error::HttpError;
use crate::retry::RetryPolicy;

use agent::agent;
pub(crate) use headers::request_with_headers;
use send::{RequestDeadline, send_with_retries};

pub type HttpResult = Result<String, HttpError>;

pub const ACCEPT_GITHUB_V3: &str = "application/vnd.github.v3+json";
pub const ACCEPT_JSON: &str = "application/json";

pub fn get_text(url: &str, config: &HttpConfig) -> HttpResult {
    get_text_with_accept(url, config, Some(ACCEPT_JSON))
}

pub fn get_text_with_accept(url: &str, config: &HttpConfig, accept: Option<&str>) -> HttpResult {
    get_text_with_accept_and_retry(url, config, accept, crate::disabled_retry_policy())
}

pub fn get_text_with_accept_and_retry(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
) -> HttpResult {
    get_text_with_accept_and_retry_inner(url, config, accept, retry_policy, None)
}

pub fn get_text_with_accept_and_retry_timeout(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
    timeout: Duration,
) -> HttpResult {
    get_text_with_accept_and_retry_inner(url, config, accept, retry_policy, Some(timeout))
}

fn get_text_with_accept_and_retry_inner(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
    timeout: Option<Duration>,
) -> HttpResult {
    let deadline = RequestDeadline::after(timeout);
    let agent = agent(config)?;
    send_with_retries("GET", retry_policy, deadline, |remaining| {
        let request = request_with_headers(agent.get(url), url, &config.auth_headers, accept);
        request_with_timeout(request, config, remaining).call()
    })
}

pub fn post_text(url: &str, body: &str, config: &HttpConfig) -> HttpResult {
    post_text_inner(url, body, config, None)
}

pub fn post_text_with_timeout(
    url: &str,
    body: &str,
    config: &HttpConfig,
    timeout: Duration,
) -> HttpResult {
    post_text_inner(url, body, config, Some(timeout))
}

fn post_text_inner(
    url: &str,
    body: &str,
    config: &HttpConfig,
    timeout: Option<Duration>,
) -> HttpResult {
    let deadline = RequestDeadline::after(timeout);
    let agent = agent(config)?;

    send_with_retries(
        "POST",
        crate::disabled_retry_policy(),
        deadline,
        |remaining| {
            let request = request_with_headers(
                agent.post(url),
                url,
                &config.auth_headers,
                Some(ACCEPT_JSON),
            )
            .header("content-type", "application/json");
            request_with_timeout(request, config, remaining).send(body)
        },
    )
}

fn request_with_timeout<B>(
    request: ureq::RequestBuilder<B>,
    config: &HttpConfig,
    remaining: Option<Duration>,
) -> ureq::RequestBuilder<B> {
    let Some(remaining) = remaining else {
        return request;
    };
    let timeout = crate::duration_from_millis(config.timeout_ms).min(remaining);
    request.config().timeout_global(Some(timeout)).build()
}

#[cfg(test)]
mod tests;
