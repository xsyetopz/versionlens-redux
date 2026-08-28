use super::*;

#[test]
fn registry_response_without_latest_creates_no_match() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("registry-response-without-latest-creates-no-match.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{"versions":{}}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
}

#[test]
fn npm_error_registry_response_creates_error_suggestion() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
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
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "error", Some("not found"));
}

#[test]
fn hex_error_registry_response_creates_error_suggestion() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
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
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "error", Some("not found"));
}

#[test]
fn hex_rate_limited_registry_response_creates_error_suggestion() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///mix.exs".to_owned(),
            "elixir".to_owned(),
            package_file_fixture("hex-rate-limited-registry-response-creates-error-suggestion.exs"),
            None,
        ),
        &[RegistryResponseInput::new(
            "plug".to_owned(),
            Hex,
            r#"{"status":429}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].status, "error");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("too many requests")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn registry_responses_try_next_matching_body_when_first_has_no_latest() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture(
                "registry-responses-try-next-matching-body-when-first-has-no-latest.json",
            ),
            None,
        ),
        &[
            RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{"versions":{}}"#.to_owned()),
            RegistryResponseInput::new(
                "left-pad".to_owned(),
                Npm,
                r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            ),
        ],
    );

    assert_update(&output, "1.1.0");
}

#[test]
fn unauthorized_registry_response_reports_auth_request_urls() {
    let (output, base_url) = crate::support::tests::with_unauthorized_server(|base_url| {
        let session = crate::support::tests::session_with_provider_settings(
            ProviderSettings {
                registry_urls: vec![RegistryUrlConfig {
                    ecosystem: Npm,
                    url: base_url.clone(),
                }],
                ..crate::default()
            },
            false,
        );
        (
            session.resolve_document(DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture(
                    "unauthorized-registry-response-reports-auth-request-urls.json",
                ),
                None,
            )),
            base_url,
        )
    });

    assert_eq!(output.authorization_required_count, 1);
    assert_eq!(output.authorization_required_requests.len(), 1);
    assert_eq!(output.authorization_required_requests[0].auth_url, base_url);
    assert_eq!(
        output.authorization_required_requests[0].request_url,
        format!("{base_url}/left-pad")
    );
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/error", name)
}
use super::assert_update;
