fn assert_single_nonvulnerable_update(output: &crate::contract::ResolveDocumentOutput) {
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.vulnerable_update_count, 0);
}

#[test]
fn apply_command_does_not_count_vulnerability_fixed_by_update() {
    let output = apply_vulnerability_case(VulnerabilityCase {
        fixture: "command-does-not-count-vulnerability-fixed-by-update.json",
        target: None,
        show_vulnerabilities: true,
        introduced: "0",
        fixed: "1.1.0",
        summary: "prototype issue",
        include_secondary: true,
    });

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.authorization_required_count, 0);
    assert_eq!(output.vulnerable_update_count, 0);
}

#[test]
fn single_apply_command_counts_vulnerable_update_targets() {
    let output = apply_vulnerability_case(VulnerabilityCase {
        fixture: "single-apply-command-counts-vulnerable-update-targets.json",
        target: Some("left-pad"),
        show_vulnerabilities: true,
        introduced: "1.1.0",
        fixed: "2.0.0",
        summary: "target issue",
        include_secondary: true,
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.authorization_required_count, 0);
    assert_eq!(output.vulnerable_update_count, 1);
    assert_eq!(
        output.vulnerable_update_package.as_deref(),
        Some("left-pad")
    );
    assert_eq!(output.vulnerable_update_version.as_deref(), Some("1.1.0"));
}

#[test]
fn bulk_apply_command_does_not_count_vulnerable_update_targets() {
    let output = apply_vulnerability_case(VulnerabilityCase {
        fixture: "bulk-apply-command-does-not-count-vulnerable-update-targets.json",
        target: None,
        show_vulnerabilities: true,
        introduced: "1.1.0",
        fixed: "2.0.0",
        summary: "target issue",
        include_secondary: false,
    });

    assert_single_nonvulnerable_update(&output);
}

#[test]
fn single_apply_command_does_not_count_vulnerable_targets_when_vulnerabilities_are_hidden() {
    let output = apply_vulnerability_case(VulnerabilityCase {
        fixture: "single-apply-command-does-not-count-vulnerable-targets-when-vulnerabilities-are-hidden.json",
        target: Some("left-pad"),
        show_vulnerabilities: false,
        introduced: "1.1.0",
        fixed: "2.0.0",
        summary: "target issue",
        include_secondary: false,
    });

    assert_single_nonvulnerable_update(&output);
}

#[test]
fn apply_command_counts_authorization_required_failures() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("command-counts-authorization-required-failures.json"), None),
        Some("update"),
        None,
        &[RegistryResponseInput::new("private-package".to_owned(), Npm, r#"{"status":401}"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 0);
    assert_eq!(output.authorization_required_count, 1);
    assert_eq!(output.vulnerable_update_count, 0);
}

#[test]
fn apply_command_does_not_count_forbidden_registry_failures_as_authorization_required() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("command-does-not-count-forbidden-registry-failures-as-authorization-required.json"), None),
        Some("update"),
        None,
        &[RegistryResponseInput::new("private-package".to_owned(), Npm, r#"{"status":403}"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 0);
    assert_eq!(output.authorization_required_count, 0);
    assert_eq!(output.vulnerable_update_count, 0);
}
struct VulnerabilityCase<'a> {
    fixture: &'a str,
    target: Option<&'a str>,
    show_vulnerabilities: bool,
    introduced: &'a str,
    fixed: &'a str,
    summary: &'a str,
    include_secondary: bool,
}

fn apply_vulnerability_case(case: VulnerabilityCase<'_>) -> ResolveDocumentOutput {
    let session = if case.show_vulnerabilities {
        standard_session()
    } else {
        session_with_vulnerability_visibility(false)
    };
    let vulnerability = serde_json::json!({
        "dist-tags": {"latest": "1.1.0"},
        "vulns": [{
            "id": "OSV-1",
            "summary": case.summary,
            "affected": [{
                "package": {"name": "left-pad"},
                "ranges": [{"events": [{"introduced": case.introduced}, {"fixed": case.fixed}]}]
            }]
        }]
    });
    let mut responses = vec![RegistryResponseInput::new(
        "left-pad".to_owned(),
        Npm,
        vulnerability.to_string(),
    )];
    if case.include_secondary {
        responses.push(RegistryResponseInput::new(
            "is-odd".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"3.0.0"}}"#.to_owned(),
        ));
    }
    session.apply_command(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture(case.fixture),
            None,
        ),
        Some("update"),
        case.target,
        &responses,
    )
}
