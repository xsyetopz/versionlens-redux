use versionlens_http::ACCEPT_GITHUB_V3;
use versionlens_parsers::Ecosystem;

use super::{accept_header_for_request, retry_policy_for_request};

#[test]
fn npm_registry_requests_omit_accept_header_like_npm_registry_fetch() {
    assert_eq!(
        accept_header_for_request(Ecosystem::Npm, "https://registry.npmjs.org/left-pad"),
        None,
    );
}

#[test]
fn github_api_requests_keep_github_accept_header() {
    assert_eq!(
        accept_header_for_request(
            Ecosystem::Npm,
            "https://api.github.com/repos/acme/repo/tags"
        ),
        Some(ACCEPT_GITHUB_V3),
    );
}

#[test]
fn npm_registry_requests_use_npm_registry_fetch_retry_policy() {
    let policy = retry_policy_for_request(Ecosystem::Npm, "https://registry.npmjs.org/left-pad");

    assert_eq!(policy.max_retries(), 2);
    assert!(policy.should_retry_status("GET", 429));
}

#[test]
fn non_npm_requests_keep_request_light_retry_behavior() {
    let policy = retry_policy_for_request(
        Ecosystem::Docker,
        "https://registry-1.docker.io/v2/library/node/tags/list",
    );

    assert_eq!(policy.max_retries(), 0);
    assert!(!policy.should_retry_status("GET", 429));
}

#[test]
fn npm_github_api_requests_keep_request_light_retry_behavior() {
    let policy = retry_policy_for_request(
        Ecosystem::Npm,
        "https://api.github.com/repos/acme/repo/tags",
    );

    assert_eq!(policy.max_retries(), 0);
    assert!(!policy.should_retry_status("GET", 429));
}
