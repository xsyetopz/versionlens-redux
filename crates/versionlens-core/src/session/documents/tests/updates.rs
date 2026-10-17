use versionlens_model::Ecosystem::Npm;

use super::{package_json_input, standard_config};
use crate::{AnalyzeDocumentOutput, RegistryResponseInput, SessionConfig, Severity};

const ASTRO: &str =
    r#"{"dist-tags":{"latest":"8.0.1"},"versions":{"7.2.2":{},"7.3.5":{},"8.0.1":{}}}"#;

async fn analyze(
    config: SessionConfig,
    manifest: &str,
    package: &str,
    body: &str,
) -> AnalyzeDocumentOutput {
    let session = crate::version_lens_session(config);
    let input = package_json_input(manifest);
    let responses = [RegistryResponseInput::new(package, Npm, body)];
    crate::support::tests::analyze_with_responses(&session, &input, &responses).await
}

fn update_diagnostics(output: &AnalyzeDocumentOutput) -> Vec<&crate::DiagnosticView> {
    output
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_deref() == Some(crate::UPDATE_AVAILABLE))
        .collect()
}

#[tokio::test]
async fn a_range_that_excludes_latest_gets_an_update_diagnostic_on_its_requirement() {
    let manifest = r#"{"dependencies":{"astro":"^7.2.2"}}"#;
    let output = analyze(standard_config(false), manifest, "astro", ASTRO).await;

    let diagnostics = update_diagnostics(&output);
    assert_eq!(diagnostics.len(), 1, "{:?}", output.diagnostics);
    let diagnostic = diagnostics[0];
    assert_eq!(diagnostic.message, "^7.2.2 → ^8.0.1");
    assert_eq!(diagnostic.severity, Severity::Information);
    assert_eq!(diagnostic.source.as_deref(), Some("VersionLens"));
    let requirement = crate::version_lens_session(standard_config(false))
        .dependencies(&package_json_input(manifest))
        .remove(0)
        .requirement_range;
    assert_eq!(diagnostic.range, requirement);
}

#[tokio::test]
async fn a_range_that_allows_latest_keeps_its_lens_and_gets_no_diagnostic() {
    let output = analyze(
        standard_config(false),
        r#"{"devDependencies":{"@biomejs/biome":"^2.5.8"}}"#,
        "@biomejs/biome",
        r#"{"dist-tags":{"latest":"2.5.14"},"versions":{"2.5.8":{},"2.5.14":{}}}"#,
    )
    .await;

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(!output.code_lenses.is_empty());
}

#[tokio::test]
async fn the_configured_severity_is_used_and_off_publishes_none() {
    let manifest = r#"{"dependencies":{"astro":"^7.2.2"}}"#;
    let mut config = standard_config(false);
    config.update_severity = Some(Severity::Hint);
    let output = analyze(config.clone(), manifest, "astro", ASTRO).await;
    assert_eq!(update_diagnostics(&output)[0].severity, Severity::Hint);

    config.update_severity = None;
    let output = analyze(config, manifest, "astro", ASTRO).await;
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[tokio::test]
async fn a_vulnerable_outdated_dependency_gets_both_diagnostics() {
    let session = crate::version_lens_session(standard_config(false));
    let input = package_json_input(r#"{"dependencies":{"left-pad":"1.0.0"}}"#);
    session
        .resolve_document_with_responses(
            input.clone(),
            &[RegistryResponseInput::new(
                "left-pad",
                Npm,
                r#"{"dist-tags":{"latest":"1.1.0"}}"#,
            )],
        )
        .await;

    let output = session.analyze_document_with_responses(
        input,
        &[crate::support::tests::npm_vulnerability_range_response(
            "left-pad",
            "OSV-1",
            "prototype issue",
            "1.1.0",
            None,
        )],
    );

    let codes = output
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code.as_deref(), diagnostic.severity))
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        [
            (Some("OSV-1"), Severity::Error),
            (Some(crate::UPDATE_AVAILABLE), Severity::Information),
        ]
    );
    assert_eq!(output.status.vulnerability_count, 1);
}

#[tokio::test]
async fn a_floor_bump_to_a_new_minor_gets_a_diagnostic_and_a_patch_bump_does_not() {
    let body = r#"{"dist-tags":{"latest":"7.3.5"},"versions":{"7.2.2":{},"7.3.5":{}}}"#;
    for (requirement, expected) in [
        ("^7.2.2", Some("^7.2.2 → ^7.3.5")),
        ("~7.2.2", Some("~7.2.2 → ~7.3.5")),
        ("^7.3.0", None),
        ("^7.3.5", None),
    ] {
        let manifest = format!(r#"{{"dependencies":{{"astro":"{requirement}"}}}}"#);
        let output = analyze(standard_config(false), &manifest, "astro", body).await;
        let messages = update_diagnostics(&output)
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>();
        assert_eq!(messages, Vec::from_iter(expected), "{requirement}");
    }
}
