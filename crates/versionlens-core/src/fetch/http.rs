use std::sync::Arc;

use versionlens_fetch::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON, ACCEPT_NPM_INSTALL_V1, Credential, HttpConfig, HttpError,
    RetryPolicy, get_text_with_accept_and_retry, get_text_with_accept_and_retry_timeout,
};
use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{Go, Maven, Npm, Python};
use versionlens_providers::http_status_message_from_code;

use crate::VersionLensSession;
use crate::concurrency::acquire_registry_permits;
use crate::error::FetchError;
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
        let accept = accept_header_for_request(ecosystem, url);
        let retry_policy = retry_policy_for_request(ecosystem, url);
        let (response, http_config) = self
            .fetch_text_with_gh_retry(url, http_config, accept, retry_policy, operation)
            .await;
        // A retry carries another credential, so its body belongs to another key.
        let cache_key = self.request_cache_key(url, &http_config);

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
        if let HttpError::RateLimited { host, reset, .. } = error {
            if let Some(source) = self.request_state.authorization.get() {
                source.rate_limited(&host, reset);
            }
            return FetchError::RateLimited { host, reset };
        }
        if let Some(status) = error.status_code()
            && let Some(message) = http_status_message_from_code(status)
        {
            if status == 401 {
                let auth_url = self.authorization_url_for_request(url);
                operation.record_authorization_request(auth_url, url.to_owned());
            }
            return FetchError::HttpStatus {
                message,
                status,
                host: url_host(url).to_owned(),
            };
        }
        let kind = http_error_kind(&error);
        crate::anyhow_error(error)
            .context(format!("failed to fetch {source} URL {url}: {kind}"))
            .into()
    }
}

async fn fetch_text(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
    operation: &OperationContext,
) -> Result<String, HttpError> {
    match operation.remaining_duration() {
        Some(remaining) => {
            get_text_with_accept_and_retry_timeout(url, config, accept, retry_policy, remaining)
                .await
        }
        None => get_text_with_accept_and_retry(url, config, accept, retry_policy).await,
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
            versionlens_fetch::npm_registry_fetch_retry_policy()
        }
        _ => versionlens_fetch::disabled_retry_policy(),
    }
}

impl VersionLensSession {
    /// Fetches `url`; when GitHub refuses it and `gh auth token` supplies a
    /// token not yet sent, retries once with it. Returns the config used last.
    async fn fetch_text_with_gh_retry(
        &self,
        url: &str,
        mut http_config: HttpConfig,
        accept: Option<&str>,
        retry_policy: RetryPolicy,
        operation: &OperationContext,
    ) -> (Result<String, HttpError>, HttpConfig) {
        let mut response = self
            .fetch_text_permitted(url, &http_config, accept, retry_policy, operation)
            .await;
        if let Err(error) = &response
            && let Some(status) = error.status_code()
        {
            let mut credential = self.credential_after_denied(url, status, operation).await;
            if credential
                .as_ref()
                .is_none_or(|credential| sent_with(&http_config, credential))
                && is_authorization_denial(error, status)
            {
                credential = self.credential_from_client(url, status, operation).await;
            }
            if let Some(credential) = credential
                && !sent_with(&http_config, &credential)
            {
                versionlens_fetch::apply_credential(&mut http_config.auth_headers, &credential);
                response = self
                    .fetch_text_permitted(url, &http_config, accept, retry_policy, operation)
                    .await;
            }
        }
        (response, http_config)
    }

    /// One request under the registry permits. The permits are held only for
    /// the request, never while a credential is being asked for, so a host
    /// that waits for the editor cannot starve the others.
    async fn fetch_text_permitted(
        &self,
        url: &str,
        http_config: &HttpConfig,
        accept: Option<&str>,
        retry_policy: RetryPolicy,
        operation: &OperationContext,
    ) -> Result<String, HttpError> {
        let Some(_permits) =
            acquire_registry_permits(self.storage_state.task_priority, operation).await
        else {
            return Err(HttpError::DeadlineExceeded);
        };
        fetch_text(url, http_config, accept, retry_policy, operation).await
    }

    /// The `gh auth token` credential, when GitHub refused `url` with `status`
    /// and no earlier source applies. Runs `gh` on a blocking thread and waits
    /// no longer than the operation deadline.
    async fn credential_after_denied(
        &self,
        url: &str,
        status: u16,
        operation: &OperationContext,
    ) -> Option<Credential> {
        let credentials = self.request_state.credentials.clone();
        let url = url.to_owned();
        let task =
            tokio::task::spawn_blocking(move || credentials.resolve_after_denied(&url, status));
        match operation.remaining_duration() {
            Some(remaining) => tokio::time::timeout(remaining, task).await.ok()?,
            None => task.await,
        }
        .ok()
        .flatten()
    }

    /// Asks the session's authorization source (the editor) for a credential
    /// for the host of `url` and keeps an answer for the session. Waits no
    /// longer than the operation deadline, and never blocks a thread.
    async fn credential_from_client(
        &self,
        url: &str,
        status: u16,
        operation: &OperationContext,
    ) -> Option<Credential> {
        let source = self.request_state.authorization.get()?;
        let host = versionlens_fetch::host_of(url)?;
        let answer = source.authorize(&host, status);
        let credential = wait_while_current(answer, operation).await??;
        self.request_state
            .credentials
            .set_client_credential(&host, credential.clone());
        Some(credential)
    }

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

/// Awaits `future` until it finishes, the operation deadline passes or the
/// operation is cancelled. `None` means it did not finish.
async fn wait_while_current<F: std::future::Future>(
    future: F,
    operation: &OperationContext,
) -> Option<F::Output> {
    let cancelled = async {
        while operation.is_current() {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    };
    let bounded = async {
        match operation.remaining_duration() {
            Some(remaining) => tokio::time::timeout(remaining, future).await.ok(),
            None => Some(future.await),
        }
    };
    tokio::select! {
        output = bounded => output,
        () = cancelled => None,
    }
}

/// Whether the host refused the request for lack of a credential. A rate
/// limit also answers 403, but it is its own error and no credential helps.
fn is_authorization_denial(error: &HttpError, status: u16) -> bool {
    matches!(error, HttpError::Client(_)) && matches!(status, 401 | 403)
}

fn sent_with(config: &HttpConfig, credential: &Credential) -> bool {
    config
        .auth_headers
        .contains(&credential.authorization_header())
}

/// The kind of a transport failure, for error lenses and logs.
fn http_error_kind(error: &HttpError) -> String {
    match error {
        HttpError::Client(error) if error.is_timeout() => "timed out".to_owned(),
        HttpError::Client(error) if error.is_connect() => "connection failed".to_owned(),
        HttpError::Client(error) if error.is_redirect() => "too many redirects".to_owned(),
        HttpError::Client(error) if error.is_decode() || error.is_body() => {
            "unreadable response body".to_owned()
        }
        HttpError::Client(error) => match error.status() {
            Some(status) => format!("HTTP {}", status.as_u16()),
            None => "request failed".to_owned(),
        },
        HttpError::Io(error) => format!("I/O error ({})", error.kind()),
        HttpError::Schema(error) => format!("invalid JSON ({error})"),
        HttpError::DeadlineExceeded
        | HttpError::ResponseTooLarge
        | HttpError::RateLimited { .. }
        | HttpError::SchemaValidation(_) => error.to_string(),
    }
}

/// The host and port of a URL, without scheme, credentials or path.
fn url_host(url: &str) -> &str {
    let rest = url.find("://").map_or(url, |index| &url[index + 3..]);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    authority.rsplit('@').next().unwrap_or(authority)
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
