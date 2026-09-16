use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread::JoinHandle;
use std::thread::spawn;
use std::time::{Duration, Instant};

use crate::config::{HttpConfig, HttpHeader};
use flate2::Compression;
use flate2::write::GzEncoder;
use reqwest::RequestBuilder;

use crate::HttpError::{Client as HttpClientError, Io as HttpIoError};

use super::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON,
    agent::tests::{uses_agent_cache, uses_same_agent_cache_key},
    get_bytes_with_accept_and_retry, get_text, get_text_with_accept_and_retry_timeout, post_text,
    request_with_headers,
};

type HttpResponseBytes = Vec<u8>;

#[tokio::test]
async fn byte_get_preserves_non_utf8_response_data() {
    let expected = vec![0, 0xff, b'\n', 0x80, 1];
    let (url, server) = serve_once(
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n2\r\n\x00\xff\r\n3\r\n\n\x80\x01\r\n0\r\n\r\n",
        Duration::ZERO,
    );

    let bytes = get_bytes_with_accept_and_retry(
        &url,
        &crate::standard_http_config(),
        Some("application/octet-stream"),
        crate::disabled_retry_policy(),
    )
    .await
    .unwrap();
    server.join().unwrap();

    assert_eq!(bytes, expected);
}

#[tokio::test]
async fn response_body_limit_rejects_oversized_content() {
    assert_response_too_large(
        b"HTTP/1.1 200 OK\r\nContent-Length: 67108865\r\nConnection: close\r\n\r\n".to_vec(),
    )
    .await;
}

#[tokio::test]
async fn negotiates_and_transparently_decompresses_gzip_responses() {
    let body = gzip(b"compressed registry response");
    let (url, request, server) = serve_once_with_request(http_response(
        &[
            ("Content-Encoding", "gzip"),
            ("Content-Type", "application/json"),
        ],
        &body,
    ));

    let result = get_text(&url, &crate::standard_http_config())
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(result, "compressed registry response");
    assert!(
        String::from_utf8(request.recv().unwrap())
            .unwrap()
            .lines()
            .any(|line| line.eq_ignore_ascii_case("accept-encoding: gzip"))
    );
}

#[tokio::test]
async fn accepts_plain_responses_when_the_server_ignores_gzip_negotiation() {
    let (url, server) = serve_once(
        http_response(&[("Content-Type", "application/json")], b"plain response"),
        Duration::ZERO,
    );

    let result = get_text(&url, &crate::standard_http_config())
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(result, "plain response");
}

#[tokio::test]
async fn decompresses_explicit_gzip_registry_archives_without_encoding_headers() {
    let body = gzip(b"compressed registry response");
    assert_archive_text("application/x-gzip", body).await;
}

async fn assert_archive_text(content_type: &str, body: HttpResponseBytes) {
    let (url, server) = serve_once(
        http_response(&[("Content-Type", content_type)], &body),
        Duration::ZERO,
    );

    let result = get_text(&url, &crate::standard_http_config())
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(result, "compressed registry response");
}

#[tokio::test]
async fn response_body_limit_applies_to_decompressed_content() {
    let body = gzip(&vec![b'x'; 64 * 1024 * 1024 + 1]);
    assert_response_too_large(http_response(&[("Content-Encoding", "gzip")], &body)).await;
}

#[tokio::test]
async fn request_timeout_is_capped_by_the_remaining_operation_budget() {
    const BUDGET: Duration = Duration::from_millis(50);
    const SERVER_DELAY: Duration = Duration::from_millis(500);

    let (url, server) = serve_once(
        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
        SERVER_DELAY,
    );
    let started = Instant::now();
    let result = get_text_with_accept_and_retry_timeout(
        &url,
        &HttpConfig {
            timeout_ms: 2_000,
            ..crate::standard_http_config()
        },
        Some(ACCEPT_JSON),
        crate::disabled_retry_policy(),
        BUDGET,
    )
    .await;
    let elapsed = started.elapsed();
    server.join().unwrap();

    assert!(matches!(result, Err(crate::HttpError::DeadlineExceeded)));
    assert!(
        elapsed < Duration::from_millis(200),
        "{BUDGET:?} transport budget took {elapsed:?}"
    );
}

#[tokio::test]
async fn invalid_url_fails_before_network() {
    let result = get_text("not a url", &crate::standard_http_config()).await;

    assert!(result.is_err());

    let result = post_text("not a url", "{}", &crate::standard_http_config()).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn configured_ca_file_is_loaded_before_requests() {
    let config = HttpConfig {
        ca_file: Some("/tmp/versionlens-missing-ca.pem".to_owned()),
        ..crate::standard_http_config()
    };

    let result = get_text("not a url", &config).await;

    assert_io_error(result);
}

#[test]
fn reuses_agents_for_equivalent_connection_settings() {
    let first_config = HttpConfig {
        auth_headers: vec![HttpHeader {
            name: "authorization".to_owned(),
            value: "Bearer first".to_owned(),
            url: None,
        }],
        ..crate::standard_http_config()
    };
    let second_config = HttpConfig {
        auth_headers: vec![HttpHeader {
            name: "authorization".to_owned(),
            value: "Bearer second".to_owned(),
            url: None,
        }],
        ..crate::standard_http_config()
    };

    assert!(uses_same_agent_cache_key(&first_config, &second_config));
}

#[test]
fn does_not_cache_agents_with_custom_tls_material() {
    let config = HttpConfig {
        ca: Some("not a pem certificate".to_owned()),
        ..crate::standard_http_config()
    };

    assert!(!uses_agent_cache(&config));
}

#[tokio::test]
async fn configured_direct_ca_pem_is_loaded_before_requests() {
    let config = HttpConfig {
        ca: Some("not a pem certificate".to_owned()),
        ..crate::standard_http_config()
    };

    let result = get_text("not a url", &config).await;

    assert_client_error(result);
}

#[tokio::test]
async fn configured_direct_client_cert_pem_is_loaded_before_requests() {
    let config = HttpConfig {
        cert: Some("not a pem certificate".to_owned()),
        key: Some("not a pem private key".to_owned()),
        ..crate::standard_http_config()
    };

    let result = get_text("not a url", &config).await;

    assert_client_error(result);
}

#[tokio::test]
async fn configured_client_cert_files_are_loaded_before_requests() {
    let config = HttpConfig {
        cert_file: Some("/tmp/versionlens-missing-client-cert.pem".to_owned()),
        key_file: Some("/tmp/versionlens-missing-client-key.pem".to_owned()),
        ..crate::standard_http_config()
    };

    let result = get_text("not a url", &config).await;

    assert_io_error(result);
}

#[test]
fn applies_versionlens_user_agent_to_request() {
    let request = request_with_headers(
        get("https://example.com"),
        "https://example.com",
        &[],
        Some(ACCEPT_JSON),
    );

    assert_eq!(
        request.build().unwrap().headers()["user-agent"],
        "versionlens-redux (github.com/xsyetopz/versionlens-redux)"
    );
}

#[test]
fn applies_default_json_accept_header_to_request() {
    let request = request_with_headers(
        get("https://example.com"),
        "https://example.com",
        &[],
        Some(ACCEPT_JSON),
    );

    assert_eq!(
        request.build().unwrap().headers()["accept"],
        "application/json"
    );
}

fn assert_io_error(result: Result<String, crate::HttpError>) {
    assert!(result.is_err());
    assert!(matches!(result, Err(HttpIoError(_))));
}

fn assert_client_error(result: Result<String, crate::HttpError>) {
    assert!(result.is_err());
    assert!(matches!(result, Err(HttpClientError(_))));
}

#[test]
fn can_omit_accept_header_for_plain_http_registry_clients() {
    let request = request_with_headers(
        get("https://proxy.golang.org/golang.org/x/mod/@v/list"),
        "https://proxy.golang.org/golang.org/x/mod/@v/list",
        &[],
        None,
    );

    assert!(!request.build().unwrap().headers().contains_key("accept"));
}

#[test]
fn applies_github_v3_accept_header_to_github_api_requests() {
    let request = request_with_headers(
        get("https://api.github.com/repos/owner/repo/tags"),
        "https://api.github.com/repos/owner/repo/tags",
        &[],
        Some(ACCEPT_GITHUB_V3),
    );

    assert_eq!(
        request.build().unwrap().headers()["accept"],
        ACCEPT_GITHUB_V3
    );
}

#[test]
fn applies_configured_headers_to_request() {
    let request = request_with_headers(
        get("https://example.com"),
        "https://example.com/package",
        &[HttpHeader {
            name: "authorization".to_owned(),
            value: "Bearer token".to_owned(),
            url: None,
        }],
        Some(ACCEPT_JSON),
    );

    assert_eq!(
        request.build().unwrap().headers()["authorization"],
        "Bearer token"
    );
}

#[test]
fn applies_url_scoped_headers_only_within_the_same_origin_and_path_scope() {
    let cases = [
        (
            "normalized host and default port",
            "https://REGISTRY.example.com:443/packages",
            "https://registry.example.com/packages/pkg?view=all",
            true,
        ),
        (
            "trailing slash",
            "https://registry.example.com/packages/",
            "https://registry.example.com/packages",
            true,
        ),
        (
            "valid child path",
            "https://registry.example.com/packages",
            "https://registry.example.com/packages/pkg",
            true,
        ),
        (
            "query does not change the path scope",
            "https://registry.example.com/packages/?scope=read",
            "https://registry.example.com/packages/pkg?version=1",
            true,
        ),
        (
            "suffix host",
            "https://registry.example.com/packages",
            "https://registry.example.com.attacker.test/packages/pkg",
            false,
        ),
        (
            "different port",
            "https://registry.example.com/packages",
            "https://registry.example.com:8443/packages/pkg",
            false,
        ),
        (
            "different scheme",
            "https://registry.example.com/packages",
            "http://registry.example.com/packages/pkg",
            false,
        ),
        (
            "path prefix without a segment boundary",
            "https://registry.example.com/packages",
            "https://registry.example.com/packages-private/pkg",
            false,
        ),
        (
            "path case differs",
            "https://registry.example.com/packages",
            "https://registry.example.com/Packages/pkg",
            false,
        ),
    ];

    for (case, auth_url, request_url, expected) in cases {
        let request = request_with_headers(
            get(request_url),
            request_url,
            &[HttpHeader {
                name: "authorization".to_owned(),
                value: "Bearer token".to_owned(),
                url: Some(auth_url.to_owned()),
            }],
            Some(ACCEPT_JSON),
        );

        assert_eq!(
            request
                .build()
                .unwrap()
                .headers()
                .contains_key("authorization"),
            expected,
            "{case}"
        );
    }
}

#[test]
fn applies_url_scoped_headers_only_to_matching_requests() {
    let request = request_with_headers(
        get("https://other.example.com"),
        "https://other.example.com/package",
        &[HttpHeader {
            name: "authorization".to_owned(),
            value: "Bearer token".to_owned(),
            url: Some("https://registry.example.com".to_owned()),
        }],
        Some(ACCEPT_JSON),
    );

    assert!(
        !request
            .build()
            .unwrap()
            .headers()
            .contains_key("authorization")
    );
}

fn get(url: &str) -> RequestBuilder {
    super::agent::install_tls_provider();
    reqwest::Client::new().get(url)
}

async fn assert_response_too_large(response: HttpResponseBytes) {
    let (url, server) = serve_once(response, Duration::ZERO);
    let result = get_text(&url, &crate::standard_http_config()).await;
    server.join().unwrap();
    assert!(matches!(result, Err(crate::HttpError::ResponseTooLarge)));
}

fn serve_once(response: impl Into<HttpResponseBytes>, delay: Duration) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let response = response.into();
    let server = spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0_u8; 1024];
        assert!(stream.read(&mut buffer).unwrap() > 0);
        std::thread::sleep(delay);
        let _ = stream.write_all(&response);
    });
    (url, server)
}

fn serve_once_with_request(
    response: HttpResponseBytes,
) -> (String, Receiver<HttpResponseBytes>, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (request_tx, request_rx) = sync_channel(1);
    let server = spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 4096];
        let length = stream.read(&mut request).unwrap();
        request_tx.send(request[..length].to_vec()).unwrap();
        stream.write_all(&response).unwrap();
    });
    (url, request_rx, server)
}

fn http_response(headers: &[(&str, &str)], body: &[u8]) -> HttpResponseBytes {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )
    .into_bytes();
    for (name, value) in headers {
        response.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
    }
    response.extend_from_slice(b"\r\n");
    response.extend_from_slice(body);
    response
}

fn gzip(body: &[u8]) -> HttpResponseBytes {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(body).unwrap();
    encoder.finish().unwrap()
}
