//! Regression fixtures for reported suggestion bugs. Each test states the
//! behavior its spec scenario requires.

use versionlens_model::{DocumentInput, Ecosystem};
use versionlens_test_support::regression;

use super::super::{ApplyCommandRequest, VersionLensSession};
use super::golden::lens_actions;
use super::test_session;
use crate::contract::{AnalyzeDocumentOutput, ResolveDocumentOutput};
use crate::{ProviderSettings, RegistryResponseInput, RegistryUrlConfig};

struct Checked {
    resolved: ResolveDocumentOutput,
    analysis: AnalyzeDocumentOutput,
    bumps: Vec<String>,
}

async fn check(
    session: &VersionLensSession,
    uri: &str,
    language: &str,
    text: &str,
    responses: &[RegistryResponseInput],
) -> Checked {
    let input = DocumentInput::new(uri, language, text, None);
    let resolved = session
        .resolve_document_with_responses(input.clone(), responses)
        .await;
    let analysis = session.analyze_document(input.clone());
    let mut bumps = Vec::new();
    for lens in &analysis.code_lenses {
        for (command, dependency_name, selected_version) in lens_actions(lens) {
            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: input.clone(),
                    command: Some(command),
                    dependency_name: Some(dependency_name),
                    selected_version,
                    responses,
                })
                .await;
            bumps.extend(output.edits.into_iter().map(|edit| edit.new_text));
        }
    }
    bumps.sort();
    bumps.dedup();
    Checked {
        resolved,
        analysis,
        bumps,
    }
}

/// A session whose registries refuse connections, so any lookup surfaces as
/// an error instead of reaching the network.
fn offline_session() -> VersionLensSession {
    let unreachable = |ecosystem| RegistryUrlConfig {
        ecosystem,
        url: "http://127.0.0.1:9/".to_owned(),
    };
    crate::test_support::session_with_provider_settings(
        ProviderSettings {
            registry_urls: vec![
                unreachable(Ecosystem::GitHub),
                unreachable(Ecosystem::Npm),
                unreachable(Ecosystem::Dotnet),
            ],
            ..Default::default()
        },
        false,
    )
}

fn titles(checked: &Checked) -> Vec<&str> {
    checked
        .analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect()
}

fn assert_status(checked: &Checked, status: &str) {
    assert!(
        titles(checked)
            .iter()
            .any(|title| title.ends_with(status) && !title.starts_with('↑')),
        "expected status `{status}`, lenses: {:?}",
        titles(checked)
    );
}

fn assert_no_bump(checked: &Checked) {
    assert!(
        titles(checked).iter().all(|title| !title.starts_with('↑')),
        "bump lens offered: {:?}",
        titles(checked)
    );
    assert_eq!(
        checked.bumps,
        Vec::<String>::new(),
        "lenses: {:?}",
        titles(checked)
    );
    assert_eq!(checked.resolved.edits, vec![]);
}

/// No error, and no diagnostic other than an available update.
fn assert_silent(checked: &Checked) {
    let noise = checked
        .analysis
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_deref() != Some(crate::UPDATE_AVAILABLE))
        .collect::<Vec<_>>();
    assert_eq!(noise, Vec::<&crate::DiagnosticView>::new());
    assert!(
        checked
            .resolved
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status != "error"),
        "suggestions: {:?}",
        checked.resolved.suggestions
    );
}

/// Checks a shared regression fixture, with the registry answers it names.
async fn check_case(session: &VersionLensSession, case: &regression::Case) -> Checked {
    let responses = case
        .registry
        .iter()
        .map(|stub| {
            let ecosystem = serde_json::from_value(serde_json::json!(stub.ecosystem))
                .unwrap_or_else(|error| panic!("{}: {error}", stub.ecosystem));
            RegistryResponseInput::new(stub.package, ecosystem, stub.body.clone())
        })
        .collect::<Vec<_>>();
    check(session, case.uri, case.language, &case.text, &responses).await
}

#[tokio::test]
async fn cargo_exact_pin_at_latest_is_current() {
    let checked = check_case(&test_session(), &regression::cargo_exact_pin()).await;
    assert_status(&checked, "latest 0.7.0");
    assert_no_bump(&checked);
}

#[tokio::test]
async fn python_exact_pin_at_latest_is_current() {
    let checked = check_case(&test_session(), &regression::python_exact_pin()).await;
    assert_status(&checked, "latest 2.34.2");
    assert_no_bump(&checked);
}

#[tokio::test]
async fn caret_range_behind_latest_offers_a_floor_bump_without_a_diagnostic() {
    let checked = check_case(&test_session(), &regression::caret_range_behind_latest()).await;
    assert_status(&checked, "satisfies latest 2.5.14");
    assert_eq!(checked.bumps, ["^2.5.14"]);
    assert_eq!(checked.analysis.diagnostics, vec![]);
}

#[tokio::test]
async fn intermediate_releases_are_not_listed_individually() {
    let checked = check_case(&test_session(), &regression::intermediate_releases()).await;
    assert_status(&checked, "satisfies 0.41.11");
    assert_eq!(checked.bumps, ["^0.41.11", "^0.42.6"]);
    for skipped in regression::STARLIGHT_SKIPPED {
        assert!(
            titles(&checked)
                .iter()
                .all(|title| !title.contains(skipped)),
            "{skipped} listed: {:?}",
            titles(&checked)
        );
    }
}

#[tokio::test]
async fn sha_pinned_action_with_floating_and_exact_tags_is_current() {
    let checked = check_case(&test_session(), &regression::sha_pinned_action()).await;
    assert_status(&checked, "latest v9.0.0");
    assert_no_bump(&checked);
}

#[tokio::test]
async fn branch_action_ref_is_fixed_without_a_lookup() {
    let checked = check_case(&offline_session(), &regression::branch_action_ref()).await;
    assert_silent(&checked);
    assert_eq!(checked.resolved.suggestions[0].status, "fixed");
    assert_no_bump(&checked);
}

#[tokio::test]
async fn vscode_engine_field_is_skipped_without_an_error() {
    let checked = check_case(&offline_session(), &regression::vscode_engine_field()).await;
    assert_silent(&checked);
}

#[tokio::test]
async fn unsupported_setup_dotnet_syntax_is_not_supported_rather_than_an_error() {
    let checked = check_case(&offline_session(), &regression::unsupported_setup_dotnet()).await;
    assert_silent(&checked);
    let dotnet = checked
        .resolved
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == "dotnet")
        .unwrap_or_else(|| panic!("dotnet not checked: {:?}", checked.resolved.suggestions));
    assert_eq!(dotnet.status, "notSupported");
}

#[tokio::test]
async fn workflow_env_reference_resolves_the_runtime() {
    let checked = check_case(&test_session(), &regression::workflow_env_reference()).await;
    assert_silent(&checked);
    let node = checked
        .resolved
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == "node")
        .unwrap_or_else(|| panic!("node not resolved: {:?}", checked.resolved.suggestions));
    assert_eq!(node.dependency.requirement, "20");
}

#[tokio::test]
async fn unresolvable_step_output_expression_is_skipped_silently() {
    let checked = check_case(&offline_session(), &regression::step_output_expression()).await;
    assert_silent(&checked);
    assert!(
        checked
            .resolved
            .suggestions
            .iter()
            .all(|suggestion| suggestion.dependency.name == "actions/setup-dotnet"),
        "expression was looked up: {:?}",
        checked.resolved.suggestions
    );
}

#[tokio::test]
async fn gradle_catalog_version_alias_resolves_through_its_library() {
    let checked = check_case(&test_session(), &regression::gradle_version_alias()).await;
    assert_silent(&checked);
    let edit = checked
        .resolved
        .edits
        .iter()
        .find(|edit| edit.new_text == "2.1.0")
        .unwrap_or_else(|| panic!("no bump: {:?}", checked.resolved));
    assert_eq!(edit.range.start.line, 1);
}

#[tokio::test]
async fn gradle_catalog_plugin_alias_uses_the_plugin_marker() {
    let checked = check_case(&test_session(), &regression::gradle_plugin_alias()).await;
    assert_silent(&checked);
    let edit = checked
        .resolved
        .edits
        .iter()
        .find(|edit| edit.new_text == "2.0.0")
        .unwrap_or_else(|| panic!("no bump: {:?}", checked.resolved));
    assert_eq!(edit.range.start.line, 1);
}

#[tokio::test]
async fn cargo_bare_version_is_a_caret_requirement() {
    let checked = check_case(&test_session(), &regression::cargo_bare_version()).await;
    assert_status(&checked, "satisfies latest 1.0.229");
    assert_eq!(checked.bumps, ["1.0.229"]);
}

#[tokio::test]
async fn package_version_fields_are_not_dependencies() {
    for case in regression::package_version_fields() {
        let uri = case.uri;
        let checked = check_case(&offline_session(), &case).await;
        assert_eq!(checked.analysis.dependencies, vec![], "{uri}");
        assert_eq!(titles(&checked), Vec::<&str>::new(), "{uri}");
        assert_eq!(checked.analysis.diagnostics, vec![], "{uri}");
    }
}
