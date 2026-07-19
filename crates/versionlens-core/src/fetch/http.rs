use std::sync::{Mutex, MutexGuard, TryLockError};
use std::thread::sleep;
use std::time::Duration;

use versionlens_http::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON, HttpError, RetryPolicy, get_text_with_accept_and_retry,
    get_text_with_accept_and_retry_timeout,
};
use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{Go, Maven, Npm, Python};
use versionlens_providers::http_status_message_from_code;

use crate::VersionLensSession;
use crate::error::FetchError;
use crate::error::FetchError::RegistryStatus as FetchRegistryStatus;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;

impl VersionLensSession {
    pub(in crate::fetch) fn get_text_or_status_with_context(
        &self,
        url: &str,
        ecosystem: Ecosystem,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<Option<String>, FetchError> {
        let http_config = self.effective_http_config(url, ecosystem, context);
        let cache_key = self.request_cache_key(url, &http_config);
        if let Some(body) = self.cached_request_body(&cache_key) {
            return Ok(Some(body));
        }
        if operation.is_expired() {
            return Err(FetchError::OperationTimeout);
        }

        let request_lock = self.request_lock(&cache_key);
        let _request_guard = lock_request_before_deadline(&request_lock, operation)?;
        if let Some(body) = self.cached_request_body(&cache_key) {
            return Ok(Some(body));
        }
        if operation.is_expired() {
            return Err(FetchError::OperationTimeout);
        }

        let accept = accept_header_for_request(ecosystem, url);
        let retry_policy = retry_policy_for_request(ecosystem, url);
        let response = match operation.remaining_duration() {
            Some(remaining) => get_text_with_accept_and_retry_timeout(
                url,
                &http_config,
                accept,
                retry_policy,
                remaining,
            ),
            None => get_text_with_accept_and_retry(url, &http_config, accept, retry_policy),
        };

        match response {
            Ok(body) => {
                if operation.is_expired() {
                    return Err(FetchError::OperationTimeout);
                }
                self.cache_request_body(cache_key, &body, ecosystem, context.manifest_kind());
                Ok(Some(body))
            }
            Err(HttpError::DeadlineExceeded) => Err(FetchError::OperationTimeout),
            Err(error) => match error.status_code().and_then(http_status_message_from_code) {
                Some(message) => {
                    if error.status_code() == Some(401) {
                        let auth_url = self.authorization_url_for_request(url);
                        operation.record_authorization_request(auth_url, url.to_owned());
                    }
                    Err(FetchRegistryStatus(message.to_owned()))
                }
                None => Err(crate::anyhow_error(error)
                    .context(format!("failed to fetch registry URL {url}"))
                    .into()),
            },
        }
    }
}

const REQUEST_LOCK_POLL_INTERVAL: Duration = Duration::from_millis(1);

fn lock_request_before_deadline<'a>(
    lock: &'a Mutex<()>,
    operation: &OperationContext,
) -> Result<MutexGuard<'a, ()>, FetchError> {
    if operation.remaining_duration().is_none() {
        return Ok(lock
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned)));
    }

    loop {
        let Some(remaining) = operation.remaining_duration() else {
            unreachable!("the operation deadline cannot be removed while waiting for a lock");
        };
        if remaining.is_zero() {
            return Err(FetchError::OperationTimeout);
        }

        match lock.try_lock() {
            Ok(guard) => return Ok(guard),
            Err(TryLockError::Poisoned(poisoned)) => {
                return Ok(crate::recover_poison(poisoned));
            }
            Err(TryLockError::WouldBlock) => sleep(remaining.min(REQUEST_LOCK_POLL_INTERVAL)),
        }
    }
}

fn accept_header_for_request(ecosystem: Ecosystem, url: &str) -> Option<&'static str> {
    if starts_with_ignore_ascii_case(url, "https://api.github.com/repos/") {
        return Some(ACCEPT_GITHUB_V3);
    }
    match ecosystem {
        Go | Maven | Npm | Python => None,
        _ => Some(ACCEPT_JSON),
    }
}

fn retry_policy_for_request(ecosystem: Ecosystem, url: &str) -> RetryPolicy {
    match ecosystem {
        Npm if !starts_with_ignore_ascii_case(url, "https://api.github.com/repos/") => {
            versionlens_http::npm_registry_fetch_retry_policy()
        }
        _ => versionlens_http::disabled_retry_policy(),
    }
}

impl VersionLensSession {
    pub(crate) fn authorization_url_for_request(&self, request_url: &str) -> String {
        self.config
            .http
            .auth_headers
            .iter()
            .filter_map(|header| header.url.as_deref())
            .map(|value| value.trim())
            .filter(|url| !url.is_empty())
            .find(|url| starts_with_ignore_ascii_case(request_url, url))
            .map(|value| value.to_owned())
            .or_else(|| url_origin(request_url))
            .unwrap_or_else(|| request_url.to_owned())
    }
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

fn url_origin(url: &str) -> Option<String> {
    let scheme_end = url.find("://")? + 3;
    let path_start = url[scheme_end..]
        .find('/')
        .map_or(url.len(), |index| scheme_end + index);
    Some(url[..path_start].to_owned())
}

#[cfg(test)]
mod tests;
