mod agent;
mod headers;
mod rate_limit;
mod send;

use std::io::Read;
use std::time::Duration;

use flate2::read::GzDecoder;

use crate::http::config::HttpConfig;
use crate::http::error::HttpError;
use crate::http::retry::RetryPolicy;

use agent::client;
pub(crate) use headers::request_with_headers;
use send::{MAX_RESPONSE_BODY_BYTES, RequestDeadline, send, send_with_retries};

/// The outcome of a request whose body is read as text.
pub type HttpResult = Result<String, HttpError>;
/// The outcome of a request whose body is read as bytes.
pub type HttpBytesResult = Result<Vec<u8>, HttpError>;

/// `Accept` value for the GitHub v3 JSON API.
pub const ACCEPT_GITHUB_V3: &str = "application/vnd.github.v3+json";
/// `Accept` value for JSON.
pub const ACCEPT_JSON: &str = "application/json";
/// `Accept` value for the abbreviated npm install metadata.
pub const ACCEPT_NPM_INSTALL_V1: &str = "application/vnd.npm.install-v1+json";

/// GETs `url` as JSON without retries and returns the body as text.
///
/// # Errors
///
/// Returns an [`HttpError`] when the request fails, times out, is rate limited or returns an error status.
pub async fn get_text(url: &str, config: &HttpConfig) -> HttpResult {
    get_text_with_accept(url, config, Some(ACCEPT_JSON)).await
}

/// GETs `url` with the given `Accept` value, without retries, and returns the body as text.
///
/// # Errors
///
/// Returns an [`HttpError`] when the request fails, times out, is rate limited or returns an error status.
pub async fn get_text_with_accept(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
) -> HttpResult {
    get_text_with_accept_and_retry(url, config, accept, crate::http::disabled_retry_policy()).await
}

/// GETs `url` with the given `Accept` value, retrying per `retry_policy`, and returns the body as text.
///
/// # Errors
///
/// Returns an [`HttpError`] when the request fails, times out, is rate limited or returns an error status.
pub async fn get_text_with_accept_and_retry(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
) -> HttpResult {
    let bytes = get_with_accept_and_retry_inner(url, config, accept, retry_policy, None).await?;
    response_text(&bytes)
}

/// Like [`get_text_with_accept_and_retry`] with `timeout` as the deadline for the whole operation, retries included.
///
/// # Errors
///
/// Returns an [`HttpError`] when the request fails, times out, is rate limited or returns an error status.
pub async fn get_text_with_accept_and_retry_timeout(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
    timeout: Duration,
) -> HttpResult {
    let bytes =
        get_with_accept_and_retry_inner(url, config, accept, retry_policy, Some(timeout)).await?;
    response_text(&bytes)
}

/// GETs `url` with the given `Accept` value, retrying per `retry_policy`, and returns the raw body.
///
/// # Errors
///
/// Returns an [`HttpError`] when the request fails, times out, is rate limited or returns an error status.
pub async fn get_bytes_with_accept_and_retry(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
) -> HttpBytesResult {
    get_with_accept_and_retry_inner(url, config, accept, retry_policy, None).await
}

/// Like [`get_bytes_with_accept_and_retry`] with `timeout` as the deadline for the whole operation, retries included.
///
/// # Errors
///
/// Returns an [`HttpError`] when the request fails, times out, is rate limited or returns an error status.
pub async fn get_bytes_with_accept_and_retry_timeout(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
    timeout: Duration,
) -> HttpBytesResult {
    get_with_accept_and_retry_inner(url, config, accept, retry_policy, Some(timeout)).await
}

async fn get_with_accept_and_retry_inner(
    url: &str,
    config: &HttpConfig,
    accept: Option<&str>,
    retry_policy: RetryPolicy,
    timeout: Option<Duration>,
) -> HttpBytesResult {
    let deadline = RequestDeadline::after(timeout);
    let client = client(config)?;
    send_with_retries("GET", retry_policy, deadline, || {
        let request = request_with_headers(client.get(url), url, &config.auth_headers, accept);
        send(request, url, config, deadline)
    })
    .await
}

/// POSTs `body` as JSON to `url` without retries and returns the response body as text.
///
/// # Errors
///
/// Returns an [`HttpError`] when the request fails, times out, is rate limited or returns an error status.
pub async fn post_text(url: &str, body: &str, config: &HttpConfig) -> HttpResult {
    post_text_inner(url, body, config, None).await
}

/// Like [`post_text`] with `timeout` as the deadline for the whole operation.
///
/// # Errors
///
/// Returns an [`HttpError`] when the request fails, times out, is rate limited or returns an error status.
pub async fn post_text_with_timeout(
    url: &str,
    body: &str,
    config: &HttpConfig,
    timeout: Duration,
) -> HttpResult {
    post_text_inner(url, body, config, Some(timeout)).await
}

async fn post_text_inner(
    url: &str,
    body: &str,
    config: &HttpConfig,
    timeout: Option<Duration>,
) -> HttpResult {
    let deadline = RequestDeadline::after(timeout);
    let client = client(config)?;
    let bytes = send_with_retries(
        "POST",
        crate::http::disabled_retry_policy(),
        deadline,
        || {
            let request = request_with_headers(
                client.post(url),
                url,
                &config.auth_headers,
                Some(ACCEPT_JSON),
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body.to_owned());
            send(request, url, config, deadline)
        },
    )
    .await?;
    response_text(&bytes)
}

fn response_text(bytes: &[u8]) -> HttpResult {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        return read_text(GzDecoder::new(bytes));
    }
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn read_text(reader: impl Read) -> HttpResult {
    let mut bytes = Vec::new();
    reader
        .take(MAX_RESPONSE_BODY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_RESPONSE_BODY_BYTES {
        return Err(HttpError::ResponseTooLarge);
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc::{Receiver, sync_channel};
    use std::thread::JoinHandle;
    use std::thread::spawn;
    use std::time::{Duration, Instant};

    use crate::http::config::{HttpConfig, HttpHeader};
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use reqwest::RequestBuilder;
    use versionlens_test_support::accept_within_deadline;

    use crate::http::HttpError::{Client as HttpClientError, Io as HttpIoError};

    use super::{
        ACCEPT_GITHUB_V3, ACCEPT_JSON,
        agent::tests::{uses_agent_cache, uses_same_agent_cache_key},
        get_bytes_with_accept_and_retry, get_text, get_text_with_accept_and_retry_timeout,
        post_text, request_with_headers,
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
            &crate::http::standard_http_config(),
            Some("application/octet-stream"),
            crate::http::disabled_retry_policy(),
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

        let result = get_text(&url, &crate::http::standard_http_config())
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

        let result = get_text(&url, &crate::http::standard_http_config())
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

        let result = get_text(&url, &crate::http::standard_http_config())
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
        let config = HttpConfig {
            timeout_ms: 2_000,
            ..crate::http::standard_http_config()
        };
        // Build and cache the client up front so the budget measures transport only.
        super::agent::client(&config).unwrap();
        let started = Instant::now();
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            get_text_with_accept_and_retry_timeout(
                &url,
                &config,
                Some(ACCEPT_JSON),
                crate::http::disabled_retry_policy(),
                BUDGET,
            ),
        )
        .await
        .expect("request ignored its operation budget");
        let elapsed = started.elapsed();
        server.join().unwrap();

        assert!(matches!(
            result,
            Err(crate::http::HttpError::DeadlineExceeded)
        ));
        assert!(
            elapsed < Duration::from_millis(200),
            "{BUDGET:?} transport budget took {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn invalid_url_fails_before_network() {
        let result = get_text("not a url", &crate::http::standard_http_config()).await;

        assert!(result.is_err());

        let result = post_text("not a url", "{}", &crate::http::standard_http_config()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn configured_ca_file_is_loaded_before_requests() {
        let config = HttpConfig {
            ca_file: Some("/tmp/versionlens-missing-ca.pem".to_owned()),
            ..crate::http::standard_http_config()
        };

        let result = get_text("not a url", &config).await;

        assert_io_error(&result);
    }

    #[test]
    fn reuses_agents_for_equivalent_connection_settings() {
        let first_config = HttpConfig {
            auth_headers: vec![HttpHeader {
                name: "authorization".to_owned(),
                value: "Bearer first".to_owned(),
                url: None,
            }],
            ..crate::http::standard_http_config()
        };
        let second_config = HttpConfig {
            auth_headers: vec![HttpHeader {
                name: "authorization".to_owned(),
                value: "Bearer second".to_owned(),
                url: None,
            }],
            ..crate::http::standard_http_config()
        };

        assert!(uses_same_agent_cache_key(&first_config, &second_config));
    }

    #[test]
    fn does_not_cache_agents_with_custom_tls_material() {
        let config = HttpConfig {
            ca: Some("not a pem certificate".to_owned()),
            ..crate::http::standard_http_config()
        };

        assert!(!uses_agent_cache(&config));
    }

    #[tokio::test]
    async fn configured_direct_ca_pem_is_loaded_before_requests() {
        let config = HttpConfig {
            ca: Some("not a pem certificate".to_owned()),
            ..crate::http::standard_http_config()
        };

        let result = get_text("not a url", &config).await;

        assert_client_error(&result);
    }

    #[tokio::test]
    async fn configured_direct_client_cert_pem_is_loaded_before_requests() {
        let config = HttpConfig {
            cert: Some("not a pem certificate".to_owned()),
            key: Some("not a pem private key".to_owned()),
            ..crate::http::standard_http_config()
        };

        let result = get_text("not a url", &config).await;

        assert_client_error(&result);
    }

    #[tokio::test]
    async fn configured_client_cert_files_are_loaded_before_requests() {
        let config = HttpConfig {
            cert_file: Some("/tmp/versionlens-missing-client-cert.pem".to_owned()),
            key_file: Some("/tmp/versionlens-missing-client-key.pem".to_owned()),
            ..crate::http::standard_http_config()
        };

        let result = get_text("not a url", &config).await;

        assert_io_error(&result);
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

    fn assert_io_error(result: &Result<String, crate::http::HttpError>) {
        assert!(result.is_err());
        assert!(matches!(result, Err(HttpIoError(_))));
    }

    fn assert_client_error(result: &Result<String, crate::http::HttpError>) {
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
        let result = get_text(&url, &crate::http::standard_http_config()).await;
        server.join().unwrap();
        assert!(matches!(
            result,
            Err(crate::http::HttpError::ResponseTooLarge)
        ));
    }

    fn serve_once(
        response: impl Into<HttpResponseBytes>,
        delay: Duration,
    ) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let response = response.into();
        let server = spawn(move || {
            let Some(mut stream) = accept_within_deadline(&listener) else {
                return;
            };
            let mut buffer = [0_u8; 1024];
            assert!(stream.read(&mut buffer).unwrap() > 0);
            std::thread::sleep(delay);
            drop(stream.write_all(&response));
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
            let Some(mut stream) = accept_within_deadline(&listener) else {
                return;
            };
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

    mod proxy {
        //! Proxy variables are process-wide, so each case runs the request in a
        //! child test process with its own environment.

        use std::process::Command;
        use std::time::Duration;

        use super::{http_response, serve_once, serve_once_with_request};

        const PROXY_VARIABLES: [&str; 8] = [
            "HTTP_PROXY",
            "http_proxy",
            "HTTPS_PROXY",
            "https_proxy",
            "ALL_PROXY",
            "all_proxy",
            "NO_PROXY",
            "no_proxy",
        ];

        /// Runs [`proxy_subprocess`] with `variables` as the only proxy settings and
        /// returns whether its request for `url` succeeded.
        fn request_in_child(url: &str, variables: &[(&str, &str)]) -> bool {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command.args([
                "--exact",
                "http::client::tests::proxy::proxy_subprocess",
                "--nocapture",
            ]);
            for variable in PROXY_VARIABLES {
                command.env_remove(variable);
            }
            command.envs(variables.iter().copied());
            command.env("VERSIONLENS_PROXY_HELPER_URL", url);
            command.status().unwrap().success()
        }

        #[tokio::test]
        async fn proxy_subprocess() {
            let Ok(url) = std::env::var("VERSIONLENS_PROXY_HELPER_URL") else {
                return;
            };
            let body = crate::http::get_text(&url, &crate::http::standard_http_config())
                .await
                .unwrap();
            assert_eq!(body, "ok");
        }

        #[test]
        fn http_proxy_routes_plain_requests_through_the_proxy() {
            let (proxy, request, server) = serve_once_with_request(http_response(&[], b"ok"));

            assert!(request_in_child(
                "http://registry.invalid/package",
                &[("HTTP_PROXY", &proxy)]
            ));
            server.join().unwrap();
            let request = String::from_utf8(request.recv().unwrap()).unwrap();
            assert!(
                request.starts_with("GET http://registry.invalid/package "),
                "{request}"
            );
        }

        #[test]
        fn lowercase_all_proxy_is_honored() {
            let (proxy, request, server) = serve_once_with_request(http_response(&[], b"ok"));

            assert!(request_in_child(
                "http://registry.invalid/package",
                &[("all_proxy", &proxy)]
            ));
            server.join().unwrap();
            assert!(request.recv().is_ok());
        }

        #[test]
        fn no_proxy_hosts_are_requested_directly() {
            let (url, server) = serve_once(http_response(&[], b"ok"), Duration::ZERO);

            // The proxy port refuses connections, so only a direct request succeeds.
            assert!(request_in_child(
                &url,
                &[
                    ("HTTP_PROXY", "http://127.0.0.1:9"),
                    ("NO_PROXY", "127.0.0.1")
                ]
            ));
            server.join().unwrap();
        }
    }
    mod rate_limit {
        use std::fmt::Write;
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        use super::serve_once;
        use crate::http::{HttpError, get_text, standard_http_config};

        fn rate_limited_response(status: &str, headers: &[(&str, String)]) -> Vec<u8> {
            let mut response =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n");
            for (name, value) in headers {
                write!(response, "{name}: {value}\r\n").unwrap();
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
            let (url, server) =
                serve_once(rate_limited_response("403 Forbidden", &[]), Duration::ZERO);
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
                    let Some(mut stream) =
                        versionlens_test_support::accept_within_deadline(&listener)
                    else {
                        return;
                    };
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
    }
}
