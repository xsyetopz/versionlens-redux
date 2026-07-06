use crate::config::{HttpConfig, HttpHeader};
use ureq::get;

use crate::HttpError::{Client as HttpClientError, Io as HttpIoError};

use super::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON,
    agent::{uses_agent_cache, uses_same_agent_cache_key},
    get_text, post_text, request_with_headers,
};

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
        "vscode-versionlens (gitlab.com/versionlens/vscode-versionlens)"
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
fn applies_url_scoped_headers_case_insensitively_like_upstream_authorizer() {
    let request = request_with_headers(
        get("https://registry.example.com/package"),
        "https://registry.example.com/package",
        &[HttpHeader {
            name: "authorization".to_owned(),
            value: "Bearer token".to_owned(),
            url: Some("https://REGISTRY.example.com".to_owned()),
        }],
        Some(ACCEPT_JSON),
    );

    assert_eq!(
        request.headers_ref().unwrap()["authorization"],
        "Bearer token"
    );
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
