mod client;
mod config;
mod error;
mod retry;
mod support;

pub use client::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON, HttpResult, get_text, get_text_with_accept,
    get_text_with_accept_and_retry, get_text_with_accept_and_retry_timeout, post_text,
    post_text_with_timeout,
};
pub use config::{
    HttpConfig, HttpConfigInput, HttpHeader, HttpHeaderInput, http_config_from_input,
    standard_http_config,
};
pub use error::HttpError;
pub use retry::{RetryPolicy, disabled_retry_policy, npm_registry_fetch_retry_policy};
#[cfg(test)]
pub(crate) use support::io_error_from_kind;
pub(crate) use support::{duration_from_millis, mutex, recover_poison};
