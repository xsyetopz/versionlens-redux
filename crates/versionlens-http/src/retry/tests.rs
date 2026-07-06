use std::io::ErrorKind::{
    AddrInUse as IoAddrInUse, ConnectionRefused as IoConnectionRefused,
    ConnectionReset as IoConnectionReset, TimedOut as IoTimedOut,
};
use ureq::Error::{
    ConnectionFailed as UreqConnectionFailed, HostNotFound as UreqHostNotFound, Io as UreqIo,
    Timeout as UreqTimeoutError,
};
use ureq::Timeout::Global as UreqTimeoutGlobal;

use super::{retry_backoff_ms, should_retry_error};

#[test]
fn request_light_parity_does_not_retry_transient_failures() {
    assert!(!should_retry_error());
    assert_eq!(retry_backoff_ms(0), 100);
    assert_eq!(retry_backoff_ms(1), 200);
}

#[test]
fn npm_registry_fetch_retry_policy_keeps_large_manifest_failures_fast() {
    let policy = crate::npm_registry_fetch_retry_policy();

    assert_eq!(policy.max_retries(), 2);
    assert_eq!(policy.retry_backoff_ms(0), Some(250));
    assert_eq!(policy.retry_backoff_ms(1), Some(500));
    assert_eq!(policy.retry_backoff_ms(2), None);
}

#[test]
fn npm_registry_fetch_retry_policy_retries_transient_non_post_statuses() {
    let policy = crate::npm_registry_fetch_retry_policy();

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
    let policy = crate::disabled_retry_policy();

    assert_eq!(policy.max_retries(), 0);
    assert_eq!(policy.retry_backoff_ms(0), None);
    assert!(!policy.should_retry_status("GET", 408));
    assert!(!policy.should_retry_status("GET", 429));
    assert!(!policy.should_retry_status("GET", 500));
}

#[test]
fn npm_registry_fetch_retry_policy_retries_transient_network_errors() {
    let policy = crate::npm_registry_fetch_retry_policy();

    assert!(
        policy.should_retry_error("GET", &UreqIo(crate::io_error_from_kind(IoConnectionReset)))
    );
    assert!(policy.should_retry_error(
        "GET",
        &UreqIo(crate::io_error_from_kind(IoConnectionRefused))
    ));
    assert!(policy.should_retry_error("GET", &UreqIo(crate::io_error_from_kind(IoAddrInUse))));
    assert!(policy.should_retry_error("GET", &UreqIo(crate::io_error_from_kind(IoTimedOut))));
    assert!(policy.should_retry_error("GET", &UreqConnectionFailed));
    assert!(policy.should_retry_error("GET", &UreqTimeoutError(UreqTimeoutGlobal)));
    assert!(!policy.should_retry_error("GET", &UreqHostNotFound));
    assert!(!policy.should_retry_error(
        "POST",
        &UreqIo(crate::io_error_from_kind(IoConnectionReset))
    ));
}
