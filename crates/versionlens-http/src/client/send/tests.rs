use std::time::Duration;

use ureq::Error::StatusCode;

use super::response::read_response_text;
use super::{RequestDeadline, send_with_retries};

#[test]
fn reads_large_registry_response_bodies() {
    let body = ureq::Body::builder().data("x".repeat(11 * 1024 * 1024));
    let response = ureq::http::Response::builder()
        .status(200)
        .body(body)
        .unwrap();

    let text = read_response_text(response).unwrap();

    assert_eq!(text.len(), 11 * 1024 * 1024);
}

#[test]
fn retries_transient_errors_when_backoff_fits_the_deadline() {
    let mut attempts = 0;

    let text = send_with_retries(
        "GET",
        crate::npm_registry_fetch_retry_policy(),
        RequestDeadline::after(Some(Duration::from_secs(1))),
        |_| {
            attempts += 1;
            if attempts == 1 {
                Err(StatusCode(503))
            } else {
                Ok(ureq::http::Response::builder()
                    .status(200)
                    .body(ureq::Body::builder().data("ok"))
                    .unwrap())
            }
        },
    )
    .unwrap();

    assert_eq!(text, "ok");
    assert_eq!(attempts, 2);
}

#[test]
fn does_not_start_retry_backoff_that_cannot_fit_the_deadline() {
    let mut attempts = 0;

    let result = send_with_retries(
        "GET",
        crate::npm_registry_fetch_retry_policy(),
        RequestDeadline::after(Some(Duration::from_millis(20))),
        |_| {
            attempts += 1;
            Err(StatusCode(503))
        },
    );

    assert!(matches!(result, Err(crate::HttpError::DeadlineExceeded)));
    assert_eq!(attempts, 1);
}
