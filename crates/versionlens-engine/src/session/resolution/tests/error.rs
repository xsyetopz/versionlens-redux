use super::*;

#[tokio::test]
async fn registry_response_without_latest_creates_no_match() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("registry-response-without-latest-creates-no-match.json"),
        None,
    );
    let response =
        RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{"versions":{}}"#.to_owned());

    let output = session
        .resolve_document_with_responses(input, &[response])
        .await;

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn npm_error_registry_response_creates_error_suggestion() {
    let session = session_without_vulnerabilities();

    let output = session
        .resolve_document_with_responses(
            DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("npm-error-registry-response-creates-error-suggestion.json"),
                None,
            ),
            &[RegistryResponseInput::new(
                "left-pad".to_owned(),
                Npm,
                r#"{"status":"E404"}"#.to_owned(),
            )],
        )
        .await;

    crate::test_support::assert_suggestion_without_edits(&output, 0, "error", Some("not found"));
}

#[tokio::test]
async fn hex_error_registry_response_creates_error_suggestion() {
    let session = session_without_vulnerabilities();

    let output = session
        .resolve_document_with_responses(
            DocumentInput::new(
                "file:///mix.exs".to_owned(),
                "elixir".to_owned(),
                package_file_fixture("hex-error-registry-response-creates-error-suggestion.exs"),
                None,
            ),
            &[RegistryResponseInput::new(
                "plug".to_owned(),
                Hex,
                r#"{"status":404}"#.to_owned(),
            )],
        )
        .await;

    crate::test_support::assert_suggestion_without_edits(&output, 0, "error", Some("not found"));
}

#[tokio::test]
async fn hex_rate_limited_registry_response_creates_rate_limited_suggestion() {
    let session = session_without_vulnerabilities();

    let output = session
        .resolve_document_with_responses(
            DocumentInput::new(
                "file:///mix.exs".to_owned(),
                "elixir".to_owned(),
                package_file_fixture(
                    "hex-rate-limited-registry-response-creates-error-suggestion.exs",
                ),
                None,
            ),
            &[RegistryResponseInput::new(
                "plug".to_owned(),
                Hex,
                r#"{"status":429}"#.to_owned(),
            )],
        )
        .await;

    assert_eq!(output.suggestions[0].status, "rateLimited");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("rate limited")
    );
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn registry_responses_try_next_matching_body_when_first_has_no_latest() {
    let session = session_without_vulnerabilities();

    let output = session
        .resolve_document_with_responses(
            DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture(
                    "registry-responses-try-next-matching-body-when-first-has-no-latest.json",
                ),
                None,
            ),
            &[
                RegistryResponseInput::new(
                    "left-pad".to_owned(),
                    Npm,
                    r#"{"versions":{}}"#.to_owned(),
                ),
                RegistryResponseInput::new(
                    "left-pad".to_owned(),
                    Npm,
                    r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
                ),
            ],
        )
        .await;

    assert_update(&output, "1.1.0");
}

#[tokio::test]
async fn unauthorized_registry_response_reports_auth_request_urls() {
    let (output, base_url) = crate::test_support::with_unauthorized_server(|base_url| async move {
        let session = crate::test_support::session_with_provider_settings(
            ProviderSettings {
                registry_urls: vec![RegistryUrlConfig {
                    ecosystem: Npm,
                    url: base_url.clone(),
                }],
                ..Default::default()
            },
            false,
        );
        (
            session
                .resolve_document(DocumentInput::new(
                    "file:///package.json".to_owned(),
                    "json".to_owned(),
                    package_file_fixture(
                        "unauthorized-registry-response-reports-auth-request-urls.json",
                    ),
                    None,
                ))
                .await,
            base_url,
        )
    })
    .await;

    assert_eq!(output.authorization_required_count, 1);
    assert_eq!(output.authorization_required_requests.len(), 1);
    assert_eq!(output.authorization_required_requests[0].auth_url, base_url);
    assert_eq!(
        output.authorization_required_requests[0].request_url,
        format!("{base_url}/left-pad")
    );
}

#[tokio::test]
async fn registry_http_status_error_names_the_status_and_host() {
    let (base_url, server) = super::github::github_api_server(vec![(404, "{}")]);
    let host = base_url
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap()
        .to_owned();
    let session = crate::test_support::session_with_provider_settings(
        ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Npm,
                url: base_url.clone(),
            }],
            ..Default::default()
        },
        false,
    );

    let output = session
        .resolve_document(DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("unauthorized-registry-response-reports-auth-request-urls.json"),
            None,
        ))
        .await;

    assert_eq!(server.join().unwrap(), ["/repos/left-pad"]);
    let expected = format!("not found (404 {host})");
    crate::test_support::assert_suggestion_without_edits(
        &output,
        0,
        "error",
        Some(expected.as_str()),
    );
}

fn package_file_fixture(name: &str) -> String {
    crate::test_support::fixture("tests/fixtures/session/resolution/tests/error", name)
}
use super::assert_update;
