use std::fmt::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::serve_once;
use crate::http::{HttpError, get_text, standard_http_config};

fn rate_limited_response(status: &str, headers: &[(&str, String)]) -> Vec<u8> {
    let mut response = format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n");
    for (name, value) in headers {
        let _ = write!(response, "{name}: {value}\r\n");
    }
    response.push_str("\r\n");
    response.into_bytes()
}

fn epoch_seconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[tokio::test]
async fn exhausted_rate_limit_holds_the_host_until_the_reset_time() {
    let reset = epoch_seconds(SystemTime::now() + Duration::from_secs(3600));
    let (url, server) = serve_once(
        rate_limited_response(
            "403 Forbidden",
            &[
                ("x-ratelimit-remaining", "0".to_owned()),
                ("x-ratelimit-reset", reset.to_string()),
            ],
        ),
        Duration::ZERO,
    );
    let config = standard_http_config();

    let first = get_text(&format!("{url}/repos/a"), &config).await;
    server.join().unwrap();
    // The server has stopped, so only the circuit can answer this request
    // with a rate limit instead of a connection error.
    let second = get_text(&format!("{url}/repos/b"), &config).await;

    for result in [first, second] {
        match result {
            Err(HttpError::RateLimited {
                host,
                status,
                reset: until,
            }) => {
                assert_eq!(host, url.trim_start_matches("http://"));
                assert_eq!(status, 403);
                assert_eq!(epoch_seconds(until), reset);
            }
            other => panic!("expected a rate limit, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn retry_after_sets_the_reset_time_of_a_429() {
    let before = SystemTime::now();
    let (url, server) = serve_once(
        rate_limited_response(
            "429 Too Many Requests",
            &[("retry-after", "120".to_owned())],
        ),
        Duration::ZERO,
    );

    let result = get_text(&url, &standard_http_config()).await;
    server.join().unwrap();

    match result {
        Err(HttpError::RateLimited { status, reset, .. }) => {
            assert_eq!(status, 429);
            assert!(reset >= before + Duration::from_secs(120), "{reset:?}");
        }
        other => panic!("expected a rate limit, got {other:?}"),
    }
}

#[tokio::test]
async fn forbidden_without_rate_limit_headers_does_not_hold_the_host() {
    let (url, server) = serve_once(rate_limited_response("403 Forbidden", &[]), Duration::ZERO);
    let config = standard_http_config();

    let first = get_text(&url, &config).await;
    server.join().unwrap();
    assert_eq!(first.unwrap_err().status_code(), Some(403));

    let (url_again, server) = serve_once(super::http_response(&[], b"ok"), Duration::ZERO);
    assert_eq!(get_text(&url_again, &config).await.unwrap(), "ok");
    server.join().unwrap();
}

/// Serves `responses` in order on one port, one per connection.
fn serve_in_order(responses: Vec<Vec<u8>>) -> (String, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0_u8; 4096];
            assert!(stream.read(&mut buffer).unwrap() > 0);
            stream.write_all(&response).unwrap();
        }
    });
    (url, server)
}

fn with_token(token: &str) -> crate::http::HttpConfig {
    crate::http::HttpConfig {
        auth_headers: vec![crate::http::HttpHeader {
            name: "authorization".to_owned(),
            value: format!("Bearer {token}"),
            url: None,
        }],
        ..standard_http_config()
    }
}

#[tokio::test]
async fn a_different_credential_is_not_held_by_another_credentials_limit() {
    let reset = epoch_seconds(SystemTime::now() + Duration::from_secs(3600));
    let (url, server) = serve_in_order(vec![
        rate_limited_response(
            "403 Forbidden",
            &[
                ("x-ratelimit-remaining", "0".to_owned()),
                ("x-ratelimit-reset", reset.to_string()),
            ],
        ),
        super::http_response(&[], b"ok"),
    ]);

    let limited = get_text(&url, &standard_http_config()).await;
    assert!(
        matches!(limited, Err(HttpError::RateLimited { .. })),
        "{limited:?}"
    );
    assert_eq!(get_text(&url, &with_token("fresh")).await.unwrap(), "ok");
    server.join().unwrap();

    // The anonymous limit still holds.
    assert!(matches!(
        get_text(&url, &standard_http_config()).await,
        Err(HttpError::RateLimited { .. })
    ));
}
