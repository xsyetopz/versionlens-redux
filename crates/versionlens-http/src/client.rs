mod agent;
mod headers;
mod send;

use std::io::Read;
use std::time::Duration;

use flate2::read::GzDecoder;

use crate::config::HttpConfig;
use crate::error::HttpError;
use crate::retry::RetryPolicy;

use agent::client;
pub(crate) use headers::request_with_headers;
use send::{MAX_RESPONSE_BODY_BYTES, RequestDeadline, send, send_with_retries};

pub type HttpResult = Result<String, HttpError>;
pub type HttpBytesResult = Result<Vec<u8>, HttpError>;

pub const ACCEPT_GITHUB_V3: &str = "application/vnd.github.v3+json";
pub const ACCEPT_JSON: &str = "application/json";
pub const ACCEPT_NPM_INSTALL_V1: &str = "application/vnd.npm.install-v1+json";

pub async fn get_text(url: &str, config: &HttpConfig) -> HttpResult {
    get_text_with_accept(url, config, Some(ACCEPT_JSON)).await
}

pub async fn get_text_with_accept(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
) -> HttpResult {
    get_text_with_accept_and_retry(url, config, accept, crate::disabled_retry_policy()).await
}

pub async fn get_text_with_accept_and_retry(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
) -> HttpResult {
    let bytes = get_with_accept_and_retry_inner(url, config, accept, retry_policy, None).await?;
    response_text(bytes)
}

pub async fn get_text_with_accept_and_retry_timeout(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
    timeout: Duration,
) -> HttpResult {
    let bytes =
        get_with_accept_and_retry_inner(url, config, accept, retry_policy, Some(timeout)).await?;
    response_text(bytes)
}

pub async fn get_bytes_with_accept_and_retry(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
) -> HttpBytesResult {
    get_with_accept_and_retry_inner(url, config, accept, retry_policy, None).await
}

pub async fn get_bytes_with_accept_and_retry_timeout(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
    timeout: Duration,
) -> HttpBytesResult {
    get_with_accept_and_retry_inner(url, config, accept, retry_policy, Some(timeout)).await
}

async fn get_with_accept_and_retry_inner(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
    timeout: Option<Duration>,
) -> HttpBytesResult {
    let deadline = RequestDeadline::after(timeout);
    let client = client(config)?;
    send_with_retries("GET", retry_policy, deadline, || {
        let request = request_with_headers(client.get(url), url, &config.auth_headers, accept);
        send(request, config, deadline)
    })
    .await
}

pub async fn post_text(url: &str, body: &str, config: &HttpConfig) -> HttpResult {
    post_text_inner(url, body, config, None).await
}

pub async fn post_text_with_timeout(
    url: &str,
    body: &str,
    config: &HttpConfig,
    timeout: Duration,
) -> HttpResult {
    post_text_inner(url, body, config, Some(timeout)).await
}

async fn post_text_inner(
    url: &str,
    body: &str,
    config: &HttpConfig,
    timeout: Option<Duration>,
) -> HttpResult {
    let deadline = RequestDeadline::after(timeout);
    let client = client(config)?;
    let bytes = send_with_retries("POST", crate::disabled_retry_policy(), deadline, || {
        let request = request_with_headers(
            client.post(url),
            url,
            &config.auth_headers,
            Some(ACCEPT_JSON),
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.to_owned());
        send(request, config, deadline)
    })
    .await?;
    response_text(bytes)
}

fn response_text(bytes: Vec<u8>) -> HttpResult {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        return read_text(GzDecoder::new(bytes.as_slice()));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_text(reader: impl Read) -> HttpResult {
    let mut bytes = Vec::new();
    reader
        .take(MAX_RESPONSE_BODY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_RESPONSE_BODY_BYTES {
        return Err(HttpError::ResponseTooLarge);
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests;
