use std::sync::Arc;

use versionlens_http::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON, ACCEPT_NPM_INSTALL_V1, HttpError, RetryPolicy,
    get_text_with_accept_and_retry, get_text_with_accept_and_retry_timeout,
};
use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{Go, Maven, Npm, Python};
use versionlens_providers::http_status_message_from_code;

use crate::VersionLensSession;
use crate::concurrency::acquire_registry_permits;
use crate::error::FetchError;
use crate::error::FetchError::RegistryStatus as FetchRegistryStatus;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;

impl VersionLensSession {
    pub(in crate::fetch) async fn get_text_or_status_with_context(
        &self,
        url: &str,
        ecosystem: Ecosystem,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<Option<Arc<str>>, FetchError> {
        let http_config = self.effective_http_config(url, ecosystem, context);
        let cache_key = self.request_cache_key(url, &http_config);
        if let Some(body) = self.cached_request_body(&cache_key) {
            return Ok(Some(body));
        }
        if operation.is_expired() {
            return Err(FetchError::OperationTimeout);
        }

        let request_lock = self.request_lock(&cache_key);
        let _request_guard = match operation.remaining_duration() {
            Some(remaining) => tokio::time::timeout(remaining, request_lock.lock())
                .await
                .map_err(|_| FetchError::OperationTimeout)?,
            None => request_lock.lock().await,
        };
        if let Some(body) = self.cached_request_body(&cache_key) {
            return Ok(Some(body));
        }
        if operation.is_expired() {
            return Err(FetchError::OperationTimeout);
        }
        let Some((_request, _background)) =
            acquire_registry_permits(self.storage_state.task_priority, operation).await
        else {
            return Err(FetchError::OperationTimeout);
        };

        let accept = accept_header_for_request(ecosystem, url);
        let retry_policy = retry_policy_for_request(ecosystem, url);
        let response = match operation.remaining_duration() {
            Some(remaining) => {
                get_text_with_accept_and_retry_timeout(
                    url,
                    &http_config,
                    accept,
                    retry_policy,
                    remaining,
                )
                .await
            }
            None => get_text_with_accept_and_retry(url, &http_config, accept, retry_policy).await,
        };

        match response {
            Ok(body) => {
                if operation.is_expired() {
                    return Err(FetchError::OperationTimeout);
                }
                let body = Arc::<str>::from(body);
                self.cache_request_body(
                    cache_key,
                    Arc::clone(&body),
                    self.cache_ttl(ecosystem, context.manifest_kind()),
                    operation,
                );
                Ok(Some(body))
            }
            Err(error) => Err(self.fetch_error_from_http(error, url, "registry", operation)),
        }
    }

    pub(in crate::fetch) fn fetch_error_from_http(
        &self,
        error: HttpError,
        url: &str,
        source: &str,
        operation: &OperationContext,
    ) -> FetchError {
        if matches!(error, HttpError::DeadlineExceeded) {
            return FetchError::OperationTimeout;
        }
        if let Some(message) = error.status_code().and_then(http_status_message_from_code) {
            if error.status_code() == Some(401) {
                let auth_url = self.authorization_url_for_request(url);
                operation.record_authorization_request(auth_url, url.to_owned());
            }
            return FetchRegistryStatus(message.to_owned());
        }
        crate::anyhow_error(error)
            .context(format!("failed to fetch {source} URL {url}"))
            .into()
    }
}

fn accept_header_for_request(ecosystem: Ecosystem, url: &str) -> Option<&'static str> {
    if starts_with_ignore_ascii_case(url, "https://api.github.com/repos/") {
        return Some(ACCEPT_GITHUB_V3);
    }
    if ecosystem == Npm && starts_with_ignore_ascii_case(url, "https://registry.npmjs.org/") {
        return Some(ACCEPT_NPM_INSTALL_V1);
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
