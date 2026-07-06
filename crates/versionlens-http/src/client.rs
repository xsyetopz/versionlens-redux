mod agent;
mod headers;
mod send;

use crate::config::HttpConfig;
use crate::error::HttpError;
use crate::retry::RetryPolicy;

use agent::agent;
pub(crate) use headers::request_with_headers;
use send::send_with_retries;

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
    let agent = agent(config)?;
    send_with_retries("GET", retry_policy, || {
        request_with_headers(agent.get(url), url, &config.auth_headers, accept).call()
    })
}

pub fn post_text(url: &str, body: &str, config: &HttpConfig) -> HttpResult {
    let agent = agent(config)?;

    send_with_retries("POST", crate::disabled_retry_policy(), || {
        request_with_headers(
            agent.post(url),
            url,
            &config.auth_headers,
            Some(ACCEPT_JSON),
        )
        .header("content-type", "application/json")
        .send(body)
    })
}

#[cfg(test)]
mod tests;
