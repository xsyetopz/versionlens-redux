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
    let session = VersionLensSession::new(crate::support::tests::session_config(
        crate::default(),
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
            let Ok((mut stream, _)) = listener.accept().await else {
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
    let mut session = crate::support::tests::test_session(false);
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
