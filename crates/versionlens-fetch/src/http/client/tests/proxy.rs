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
