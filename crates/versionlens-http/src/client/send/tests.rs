use std::time::Duration;

use ureq::Error::StatusCode;

use super::response::{read_response_bytes, read_response_text};
use super::{RequestDeadline, send_with_retries};

fn retry_fixture<T>(
    body: Vec<u8>,
    read_response: impl FnMut(super::HttpResponse) -> Result<T, crate::HttpError>,
) -> (T, usize) {
    let mut attempts = 0;
    let response = send_with_retries(
        "GET",
        crate::npm_registry_fetch_retry_policy(),
        RequestDeadline::after(Some(Duration::from_secs(1))),
        read_response,
        |_| {
            attempts += 1;
            if attempts == 1 {
                Err(StatusCode(503))
            } else {
                Ok(ureq::http::Response::builder()
                    .status(200)
                    .body(ureq::Body::builder().data(body.clone()))
                    .unwrap())
            }
        },
    )
    .unwrap();
    (response, attempts)
}

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
    let (text, attempts) = retry_fixture(b"ok".to_vec(), read_response_text);

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
        read_response_text,
        |_| {
            attempts += 1;
            Err(StatusCode(503))
        },
    );

    assert!(matches!(result, Err(crate::HttpError::DeadlineExceeded)));
    assert_eq!(attempts, 1);
}

#[test]
fn byte_response_preserves_non_utf8_data_across_a_retry() {
    let expected = vec![0, 0xff, b'\n', 0x80, 1];
    let (bytes, attempts) = retry_fixture(expected.clone(), read_response_bytes);

    assert_eq!(bytes, expected);
    assert_eq!(attempts, 2);
}

#[test]
fn byte_response_rejects_bodies_above_the_shared_limit() {
    const TOO_LARGE: u64 = 64 * 1024 * 1024 + 1;
    let body = ureq::Body::builder()
        .limit(TOO_LARGE)
        .reader(std::io::repeat(0));
    let response = ureq::http::Response::builder()
        .status(200)
        .body(body)
        .unwrap();

    assert!(read_response_bytes(response).is_err());
}
