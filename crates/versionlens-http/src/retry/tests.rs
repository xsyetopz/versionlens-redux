use std::io::{Error, ErrorKind};

use super::{RetryPolicy, retry_backoff_ms, should_retry_error};

#[test]
fn request_light_parity_does_not_retry_transient_failures() {
    assert!(!should_retry_error(&ureq::Error::StatusCode(408)));
    assert!(!should_retry_error(&ureq::Error::StatusCode(429)));
    assert!(!should_retry_error(&ureq::Error::StatusCode(500)));
    assert!(!should_retry_error(&ureq::Error::StatusCode(404)));
    assert!(!should_retry_error(&ureq::Error::BadUri(
        "not a url".to_owned()
    )));
    assert_eq!(retry_backoff_ms(0), 100);
    assert_eq!(retry_backoff_ms(1), 200);
}

#[test]
fn npm_registry_fetch_retry_policy_keeps_large_manifest_failures_fast() {
    let policy = RetryPolicy::npm_registry_fetch();

    assert_eq!(policy.max_retries(), 2);
    assert_eq!(policy.retry_backoff_ms(0), Some(250));
    assert_eq!(policy.retry_backoff_ms(1), Some(500));
    assert_eq!(policy.retry_backoff_ms(2), None);
}

#[test]
fn npm_registry_fetch_retry_policy_retries_transient_non_post_statuses() {
    let policy = RetryPolicy::npm_registry_fetch();

    assert!(policy.should_retry_status("GET", 408));
    assert!(policy.should_retry_status("GET", 420));
    assert!(policy.should_retry_status("GET", 429));
    assert!(policy.should_retry_status("GET", 500));
    assert!(policy.should_retry_status("GET", 503));
    assert!(!policy.should_retry_status("GET", 404));
    assert!(!policy.should_retry_status("POST", 500));
}

#[test]
fn disabled_retry_policy_preserves_request_light_no_retry_behavior() {
    let policy = RetryPolicy::disabled();

    assert_eq!(policy.max_retries(), 0);
    assert_eq!(policy.retry_backoff_ms(0), None);
    assert!(!policy.should_retry_status("GET", 408));
    assert!(!policy.should_retry_status("GET", 429));
    assert!(!policy.should_retry_status("GET", 500));
}

#[test]
fn npm_registry_fetch_retry_policy_retries_transient_network_errors() {
    let policy = RetryPolicy::npm_registry_fetch();

    assert!(policy.should_retry_error(
        "GET",
        &ureq::Error::Io(Error::from(ErrorKind::ConnectionReset))
    ));
    assert!(policy.should_retry_error(
        "GET",
        &ureq::Error::Io(Error::from(ErrorKind::ConnectionRefused))
    ));
    assert!(policy.should_retry_error("GET", &ureq::Error::Io(Error::from(ErrorKind::AddrInUse))));
    assert!(policy.should_retry_error("GET", &ureq::Error::Io(Error::from(ErrorKind::TimedOut))));
    assert!(policy.should_retry_error("GET", &ureq::Error::ConnectionFailed));
    assert!(policy.should_retry_error("GET", &ureq::Error::Timeout(ureq::Timeout::Global)));
    assert!(!policy.should_retry_error("GET", &ureq::Error::HostNotFound));
    assert!(!policy.should_retry_error(
        "POST",
        &ureq::Error::Io(Error::from(ErrorKind::ConnectionReset))
    ));
}
