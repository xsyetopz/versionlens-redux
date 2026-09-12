use crate::RegistryResponseInput;
use crate::SessionConfig;
use versionlens_model::DocumentInput;

use super::OPERATION_TIMEOUT_MESSAGE;

use versionlens_model::Ecosystem::Npm;

fn batched_fixture_input() -> DocumentInput {
    DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("batched-resolution-preserves-dependency-order.json"),
        None,
    )
}

fn batched_fixture_responses() -> Vec<RegistryResponseInput> {
    [
        "dep-00", "dep-01", "dep-02", "dep-03", "dep-04", "dep-05", "dep-06", "dep-07", "dep-08",
        "dep-09", "dep-10", "dep-11",
    ]
    .into_iter()
    .map(|name| {
        RegistryResponseInput::new(
            name.to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        )
    })
    .collect()
}

#[test]
fn batched_resolution_preserves_dependency_order() {
    let session = crate::support::tests::test_session(false);
    let responses = batched_fixture_responses();
    let names = responses
        .iter()
        .map(|response| response.package.as_str())
        .collect::<Vec<_>>();

    let output = session.resolve_document_with_responses(batched_fixture_input(), &responses);
    let resolved_names = output
        .suggestions
        .iter()
        .map(|suggestion| suggestion.dependency.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(resolved_names, names);
}

#[test]
fn expired_operation_returns_errors_without_resolving_dependencies() {
    let mut http = versionlens_http::standard_http_config();
    http.timeout_ms = 0;
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http,
    });
    let responses = batched_fixture_responses();

    let output = session.resolve_document_with_responses(batched_fixture_input(), &responses);

    assert_eq!(output.suggestions.len(), responses.len());
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status == "error")
    );
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.latest.as_deref() == Some(OPERATION_TIMEOUT_MESSAGE))
    );
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/parallel/tests", name)
}
