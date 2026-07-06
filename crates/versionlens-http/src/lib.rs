#[cfg(test)]
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::sync::{Mutex as StdMutex, PoisonError as SyncPoisonError};
use std::time::Duration as StdDuration;
mod client;
mod config;
mod error;
mod retry;

pub use client::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON, HttpResult, get_text, get_text_with_accept,
    get_text_with_accept_and_retry, post_text,
};
pub use config::{HttpConfig, HttpConfigInput, HttpHeader, HttpHeaderInput, standard_http_config};
pub use error::HttpError;
pub use retry::RetryPolicy;

pub fn disabled_retry_policy() -> RetryPolicy {
    retry::disabled_retry_policy()
}

pub fn npm_registry_fetch_retry_policy() -> RetryPolicy {
    retry::npm_registry_fetch_retry_policy()
}

pub(crate) fn recover_poison<T>(poisoned: SyncPoisonError<T>) -> T {
    poisoned.into_inner()
}

pub(crate) const fn duration_from_millis(milliseconds: u64) -> StdDuration {
    std::time::Duration::from_millis(milliseconds)
}

pub(crate) fn mutex<T>(value: T) -> StdMutex<T> {
    std::sync::Mutex::new(value)
}

pub fn http_config_from_input(input: HttpConfigInput) -> HttpConfig {
    config::http_config_from_input(input)
}

#[cfg(test)]
pub(crate) fn io_error_from_kind(kind: IoErrorKind) -> IoError {
    kind.into()
}
