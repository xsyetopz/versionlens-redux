use std::time::{Duration, Instant};

use versionlens_fetch::{ACCEPT_GITHUB_V3, ACCEPT_NPM_INSTALL_V1};

use super::{accept_header_for_request, retry_policy_for_request};
use crate::VersionLensSession;
use crate::registry::RegistryContext;
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
