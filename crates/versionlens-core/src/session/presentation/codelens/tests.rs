use crate::RegistryResponseInput;
use versionlens_model::DocumentInput;

use crate::{AnalyzeDocumentOutput, SessionConfig, SuggestionIndicators, VersionLensSession};
use versionlens_model::Ecosystem::{Docker, Npm, Python};

mod actions;
mod docker;
mod npm;
mod python;
mod ranges;
mod vulnerabilities;

include!("tests/indicators.rs");
include!("tests/updates.rs");
include!("tests/status.rs");

fn npm_response(package: &str, latest: &str) -> RegistryResponseInput {
    RegistryResponseInput::new(
        package.to_owned(),
        Npm,
        format!(r#"{{"dist-tags":{{"latest":"{latest}"}}}}"#),
    )
}

fn npm_versions_response(latest: &str, versions: &[&str]) -> RegistryResponseInput {
    let versions = versions
        .iter()
        .map(|version| (*version, serde_json::json!({})))
        .collect::<std::collections::HashMap<_, _>>();
    RegistryResponseInput::new(
        "left-pad".to_owned(),
        Npm,
        serde_json::json!({"dist-tags": {"latest": latest}, "versions": versions}).to_string(),
    )
}

fn build_code_lens_output(fixture: &str, latest: &str, versions: &[&str]) -> AnalyzeDocumentOutput {
    let session = standard_session();
    let input = package_document(fixture);
    session
        .resolve_document_with_responses(input.clone(), &[npm_versions_response(latest, versions)]);
    session.analyze_document(input)
}

fn npm_vulnerability_response(
    latest: Option<&str>,
    versions: &[&str],
    id: &str,
    summary: &str,
    affected_range: Option<(&str, &str)>,
) -> RegistryResponseInput {
    let mut affected = serde_json::json!({"package": {"name": "left-pad"}});
    if let Some((introduced, fixed)) = affected_range {
        affected["ranges"] = serde_json::json!([{
            "events": [{"introduced": introduced}, {"fixed": fixed}]
        }]);
    } else {
        affected["versions"] = serde_json::json!(versions);
    }

    let mut body = serde_json::json!({
        "vulns": [{
            "id": id,
            "summary": summary,
            "affected": [affected]
        }]
    });
    if let Some(latest) = latest {
        body["dist-tags"] = serde_json::json!({"latest": latest});
    }
    if !versions.is_empty() {
        body["versions"] = serde_json::json!(
            versions
                .iter()
                .map(|version| (*version, serde_json::json!({})))
                .collect::<std::collections::HashMap<_, _>>()
        );
    }
    RegistryResponseInput::new("left-pad".to_owned(), Npm, body.to_string())
}

fn session_with_empty_vulnerable_indicator() -> VersionLensSession {
    let mut indicators = test_indicators();
    indicators.updateable_vulnerable.clear();
    session_with_indicators(indicators, true)
}

fn session_with_indicators(
    indicators: SuggestionIndicators,
    show_vulnerabilities: bool,
) -> VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: indicators,
        show_vulnerabilities,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}

fn test_indicators() -> SuggestionIndicators {
    SuggestionIndicators {
        latest: "L".to_owned(),
        satisfies_latest: "S".to_owned(),
        directory: "D".to_owned(),
        error: "E".to_owned(),
        no_match: "N".to_owned(),
        matched: "M".to_owned(),
        downgradeable: "D".to_owned(),
        updateable: "U".to_owned(),
        updateable_vulnerable: "V".to_owned(),
        build: "B".to_owned(),
    }
}

fn standard_session() -> VersionLensSession {
    session_with_indicators(test_indicators(), true)
}

fn package_document(fixture: &str) -> DocumentInput {
    DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture(fixture),
        None,
    )
}

fn analyze_npm_fixture_with_response(
    session: &crate::VersionLensSession,
    fixture: &str,
    response: &str,
) -> AnalyzeDocumentOutput {
    let input = package_document(fixture);
    crate::support::tests::analyze_with_responses(
        session,
        &input,
        &[crate::RegistryResponseInput::new(
            "left-pad".to_owned(),
            versionlens_model::Ecosystem::Npm,
            response.to_owned(),
        )],
    )
}

fn lens_titles(output: &AnalyzeDocumentOutput) -> Vec<&str> {
    output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect()
}

fn lens_commands(output: &AnalyzeDocumentOutput) -> Vec<&str> {
    output
        .code_lenses
        .iter()
        .map(|lens| lens.command.as_str())
        .collect()
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/presentation/codelens", name)
}
