use std::sync::mpsc::sync_channel;
use std::thread::spawn;
use std::time::{Duration, Instant};

use versionlens_http::ACCEPT_GITHUB_V3;

use super::{accept_header_for_request, lock_request_before_deadline, retry_policy_for_request};
use crate::error::FetchError;
use crate::session::operation::OperationContext;
use versionlens_model::Ecosystem::{Docker, Npm};

#[test]
fn npm_registry_requests_omit_accept_header_like_npm_registry_fetch() {
    assert_eq!(
        accept_header_for_request(Npm, "https://registry.npmjs.org/left-pad"),
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
fn contended_request_lock_stops_at_the_operation_deadline() {
    const BUDGET: Duration = Duration::from_millis(30);
    const WATCHDOG: Duration = Duration::from_millis(500);

    let request_lock = crate::arc(crate::mutex(()));
    let held_guard = request_lock.lock().unwrap();
    let waiting_lock = crate::clone_arc(&request_lock);
    let (result_tx, result_rx) = sync_channel(1);
    let started = Instant::now();
    let waiter = spawn(move || {
        let operation = OperationContext::with_timeout(BUDGET);
        let result = lock_request_before_deadline(&waiting_lock, &operation);
        result_tx
            .send(matches!(result, Err(FetchError::OperationTimeout)))
            .unwrap();
    });

    let timed_out = result_rx
        .recv_timeout(WATCHDOG)
        .expect("request lock wait exceeded its watchdog");
    let elapsed = started.elapsed();
    drop(held_guard);
    waiter.join().unwrap();

    assert!(timed_out);
    assert!(elapsed >= BUDGET);
    assert!(
        elapsed < Duration::from_millis(150),
        "{BUDGET:?} request-lock budget took {elapsed:?}"
    );
}
