use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::sync_channel;
use std::thread::spawn;
use std::time::{Duration, Instant};

use crate::config::{HttpConfig, HttpHeader};
use ureq::get;

use crate::HttpError::{Client as HttpClientError, Io as HttpIoError};

use super::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON,
    agent::{uses_agent_cache, uses_same_agent_cache_key},
    get_text, get_text_with_accept_and_retry_timeout, post_text, request_with_headers,
};

#[test]
fn request_timeout_is_capped_by_the_remaining_operation_budget() {
    const BUDGET: Duration = Duration::from_millis(50);
    const SERVER_DELAY: Duration = Duration::from_millis(500);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (request_tx, request_rx) = sync_channel(1);
    let (release_tx, release_rx) = sync_channel(1);
    let server = spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0_u8; 1024];
        let bytes_read = stream.read(&mut buffer).unwrap();
        assert!(bytes_read > 0);
        request_tx.send(()).unwrap();
        let _ = release_rx.recv_timeout(SERVER_DELAY);
        let _ = stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
    });
    let client = spawn(move || {
        get_text_with_accept_and_retry_timeout(
            &url,
            &HttpConfig {
                timeout_ms: 2_000,
                ..crate::standard_http_config()
            },
            Some(ACCEPT_JSON),
            crate::disabled_retry_policy(),
            BUDGET,
        )
    });

    request_rx.recv().unwrap();
    let started = Instant::now();
    let result = client.join().unwrap();
    let elapsed = started.elapsed();
    let _ = release_tx.send(());
    server.join().unwrap();

    assert!(matches!(result, Err(crate::HttpError::DeadlineExceeded)));
    assert!(
        elapsed < Duration::from_millis(200),
        "{BUDGET:?} transport budget took {elapsed:?}"
    );
}

#[test]
fn invalid_url_fails_before_network() {
    let result = get_text("not a url", &crate::standard_http_config());

    assert!(result.is_err());

    let result = post_text("not a url", "{}", &crate::standard_http_config());

    assert!(result.is_err());
}

#[test]
fn configured_ca_file_is_loaded_before_requests() {
    let config = HttpConfig {
        ca_file: Some("/tmp/versionlens-missing-ca.pem".to_owned()),
        ..crate::standard_http_config()
    };

    let result = get_text("not a url", &config);

    assert!(result.is_err());
    assert!(matches!(result, Err(HttpIoError(_))));
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

#[test]
fn configured_direct_ca_pem_is_loaded_before_requests() {
    let config = HttpConfig {
        ca: Some("not a pem certificate".to_owned()),
        ..crate::standard_http_config()
    };

    let result = get_text("not a url", &config);

    assert!(result.is_err());
    assert!(matches!(result, Err(HttpClientError(_))));
}

#[test]
fn configured_direct_client_cert_pem_is_loaded_before_requests() {
    let config = HttpConfig {
        cert: Some("not a pem certificate".to_owned()),
        key: Some("not a pem private key".to_owned()),
        ..crate::standard_http_config()
    };

    let result = get_text("not a url", &config);

    assert!(result.is_err());
    assert!(matches!(result, Err(HttpClientError(_))));
}

#[test]
fn configured_client_cert_files_are_loaded_before_requests() {
    let config = HttpConfig {
        cert_file: Some("/tmp/versionlens-missing-client-cert.pem".to_owned()),
        key_file: Some("/tmp/versionlens-missing-client-key.pem".to_owned()),
        ..crate::standard_http_config()
    };

    let result = get_text("not a url", &config);

    assert!(result.is_err());
    assert!(matches!(result, Err(HttpIoError(_))));
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
        request.headers_ref().unwrap()["user-agent"],
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

    assert_eq!(request.headers_ref().unwrap()["accept"], "application/json");
}

#[test]
fn can_omit_accept_header_for_plain_http_registry_clients() {
    let request = request_with_headers(
        get("https://proxy.golang.org/golang.org/x/mod/@v/list"),
        "https://proxy.golang.org/golang.org/x/mod/@v/list",
        &[],
        None,
    );

    assert!(!request.headers_ref().unwrap().contains_key("accept"));
}

#[test]
fn applies_github_v3_accept_header_to_github_api_requests() {
    let request = request_with_headers(
        get("https://api.github.com/repos/owner/repo/tags"),
        "https://api.github.com/repos/owner/repo/tags",
        &[],
        Some(ACCEPT_GITHUB_V3),
    );

    assert_eq!(request.headers_ref().unwrap()["accept"], ACCEPT_GITHUB_V3);
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
        request.headers_ref().unwrap()["authorization"],
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
            request.headers_ref().unwrap().contains_key("authorization"),
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

    assert!(!request.headers_ref().unwrap().contains_key("authorization"));
}
