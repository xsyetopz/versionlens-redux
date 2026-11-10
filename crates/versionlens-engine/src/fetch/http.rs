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
        anyhow::Error::from(error)
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
mod tests {
use crate::ProviderSettings;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use versionlens_fetch::{ACCEPT_GITHUB_V3, ACCEPT_NPM_INSTALL_V1};

use super::{accept_header_for_request, retry_policy_for_request};
use crate::VersionLensSession;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;
use versionlens_model::Ecosystem::*;

#[test]
fn official_npm_registry_requests_use_abbreviated_metadata() {
    assert_eq!(
        accept_header_for_request(Npm, "https://registry.npmjs.org/left-pad"),
        Some(ACCEPT_NPM_INSTALL_V1),
    );
}

#[test]
fn custom_npm_registry_requests_keep_exact_response_contract() {
    assert_eq!(
        accept_header_for_request(Npm, "https://registry.example.test/left-pad"),
        None,
    );
}

#[test]
fn github_api_requests_keep_github_accept_header() {
    assert_eq!(
        accept_header_for_request(Npm, "https://api.github.com/repos/acme/repo/tags"),
        Some(ACCEPT_GITHUB_V3),
    );
}

#[test]
fn npm_registry_requests_use_npm_registry_fetch_retry_policy() {
    let policy = retry_policy_for_request(Npm, "https://registry.npmjs.org/left-pad");

    assert_eq!(policy.max_retries(), 2);
    assert!(policy.should_retry_status("GET", 429));
}

#[test]
fn non_npm_requests_keep_request_light_retry_behavior() {
    let policy = retry_policy_for_request(
        Docker,
        "https://registry-1.docker.io/v2/library/node/tags/list",
    );

    assert_eq!(policy.max_retries(), 0);
    assert!(!policy.should_retry_status("GET", 429));
}

#[test]
fn npm_github_api_requests_keep_request_light_retry_behavior() {
    let policy = retry_policy_for_request(Npm, "https://api.github.com/repos/acme/repo/tags");

    assert_eq!(policy.max_retries(), 0);
    assert!(!policy.should_retry_status("GET", 429));
}

#[test]
fn client_credential_replaces_config_authorization_for_github_requests() {
    let session = VersionLensSession::new(crate::test_support::session_config(
        ProviderSettings::default(),
        false,
    ));
    session.credentials().set_client_credential(
        "api.github.com",
        versionlens_fetch::Credential::new("Bearer", "client-token"),
    );
    let url = "https://api.github.com/repos/acme/repo/tags";

    let config = session.effective_http_config(url, Npm, &RegistryContext::default());

    let values: Vec<_> = config
        .auth_headers
        .iter()
        .filter(|header| header.name.eq_ignore_ascii_case("authorization"))
        .map(|header| header.value.as_str())
        .collect();
    assert_eq!(values, ["Bearer client-token"]);
    assert!(!format!("{config:?}").contains("client-token"));
}

const GH_TOKEN: &str = "gho_MockGhTokenValue456";

/// Serves 200 `[]` to requests that carry `Bearer {GH_TOKEN}` when `accept_gh`
/// is set, and 401 to every other. Returns the base URL, the `Authorization`
/// header of each request in order, and the server task.
async fn serve_github(
    accept_gh: bool,
) -> (
    String,
    Arc<Mutex<Vec<Option<String>>>>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let task = tokio::spawn(async move {
        loop {
            let Ok(Ok((mut stream, _))) =
                tokio::time::timeout(Duration::from_secs(5), listener.accept()).await
            else {
                return;
            };
            let mut buffer = vec![0_u8; 8192];
            let read = stream.read(&mut buffer).await.unwrap_or(0);
            let request = String::from_utf8_lossy(&buffer[..read]).into_owned();
            let auth = request
                .lines()
                .find_map(|line| line.strip_prefix("authorization: "))
                .map(str::to_owned);
            let granted = accept_gh && auth.as_deref() == Some(&format!("Bearer {GH_TOKEN}"));
            log.lock().unwrap().push(auth);
            let response = if granted {
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]"
            } else {
                "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            };
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    (url, seen, task)
}

fn session_with_gh(gh_token: &str) -> (VersionLensSession, Arc<AtomicUsize>) {
    let mut session = crate::test_support::test_session(false);
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let gh_token = gh_token.to_owned();
    session.request_state.credentials = versionlens_fetch::CredentialChain::with_sources(
        |_| None,
        move || {
            counter.fetch_add(1, Ordering::SeqCst);
            Some(gh_token.clone())
        },
    )
    .with_github_host("127.0.0.1");
    (session, calls)
}

async fn fetch(
    session: &VersionLensSession,
    url: &str,
) -> Result<Option<Arc<str>>, crate::error::FetchError> {
    session
        .get_text_or_status_with_context(
            url,
            Go,
            &RegistryContext::default(),
            &OperationContext::default(),
        )
        .await
}

#[tokio::test]
async fn denied_github_request_retries_with_the_gh_token_and_caches_that_result() {
    let (url, seen, server) = serve_github(true).await;
    let (session, gh_calls) = session_with_gh(GH_TOKEN);
    let url = format!("{url}/repos/acme/repo/tags");

    let body = fetch(&session, &url).await.unwrap();
    assert_eq!(body.as_deref(), Some("[]"));
    assert_eq!(
        *seen.lock().unwrap(),
        [None, Some(format!("Bearer {GH_TOKEN}"))]
    );

    // The retried body is cached under the authenticated key, so this
    // request is answered without reaching the server.
    server.abort();
    let again = fetch(&session, &url).await.unwrap();
    assert_eq!(again.as_deref(), Some("[]"));
    assert_eq!(seen.lock().unwrap().len(), 2);
    assert_eq!(gh_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn second_denial_neither_reruns_gh_nor_resends_the_same_token() {
    let (url, seen, server) = serve_github(false).await;
    let (session, gh_calls) = session_with_gh(GH_TOKEN);
    let url = format!("{url}/repos/acme/repo/tags");

    assert!(fetch(&session, &url).await.is_err());
    assert_eq!(seen.lock().unwrap().len(), 2, "request plus one retry");
    assert!(fetch(&session, &url).await.is_err());
    server.abort();

    // The second call already sent the gh token, so a 401 gives no retry.
    assert_eq!(seen.lock().unwrap().len(), 3);
    assert_eq!(gh_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn denied_github_errors_do_not_carry_the_token() {
    let (url, _seen, server) = serve_github(false).await;
    let (session, _) = session_with_gh(GH_TOKEN);
    let url = format!("{url}/repos/acme/repo/tags");

    let error = fetch(&session, &url).await.unwrap_err();
    server.abort();

    let text = format!("{error} {error:?} {error:#} {error:#?}");
    assert!(text.contains("401") || text.to_lowercase().contains("unauthorized"));
    assert!(!text.contains(GH_TOKEN), "token leaked: {text}");
}

#[tokio::test]
async fn contended_request_lock_stops_at_the_operation_deadline() {
    const BUDGET: Duration = Duration::from_millis(30);
    const WATCHDOG: Duration = Duration::from_millis(500);

    let request_lock = std::sync::Arc::new(tokio::sync::Mutex::new(()));
    let held_guard = request_lock.lock().await;
    let started = Instant::now();
    let timed_out = tokio::time::timeout(BUDGET, request_lock.lock())
        .await
        .is_err();
    let elapsed = started.elapsed();
    drop(held_guard);

    assert!(timed_out);
    assert!(elapsed >= BUDGET);
    assert!(
        elapsed < WATCHDOG,
        "{BUDGET:?} request-lock budget took {elapsed:?}"
    );
}
}
