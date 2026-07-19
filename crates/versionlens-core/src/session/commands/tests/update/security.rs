#[test]
fn apply_command_does_not_count_vulnerability_fixed_by_update() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-does-not-count-vulnerability-fixed-by-update.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Npm,
                body: r#"{
                  "dist-tags": { "latest": "1.1.0" },
                  "vulns": [{
                    "id": "OSV-1",
                    "summary": "prototype issue",
                    "affected": [{
                      "package": { "name": "left-pad" },
                      "ranges": [{
                        "events": [{ "introduced": "0" }, { "fixed": "1.1.0" }]
                      }]
                    }]
                  }]
                }"#
                .to_owned(),
            },
            RegistryResponseInput {
                package: "is-odd".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"3.0.0"}}"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.authorization_required_count, 0);
    assert_eq!(output.vulnerable_update_count, 0);
}

#[test]
fn single_apply_command_counts_vulnerable_update_targets() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "single-apply-command-counts-vulnerable-update-targets.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("left-pad"),
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Npm,
                body: r#"{
                  "dist-tags": { "latest": "1.1.0" },
                  "vulns": [{
                    "id": "OSV-1",
                    "summary": "target issue",
                    "affected": [{
                      "package": { "name": "left-pad" },
                      "ranges": [{
                        "events": [{ "introduced": "1.1.0" }, { "fixed": "2.0.0" }]
                      }]
                    }]
                  }]
                }"#
                .to_owned(),
            },
            RegistryResponseInput {
                package: "is-odd".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"3.0.0"}}"#.to_owned(),
            },
        ],
    );

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
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "bulk-apply-command-does-not-count-vulnerable-update-targets.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "1.1.0" },
              "vulns": [{
                "id": "OSV-1",
                "summary": "target issue",
                "affected": [{
                  "package": { "name": "left-pad" },
                  "ranges": [{
                    "events": [{ "introduced": "1.1.0" }, { "fixed": "2.0.0" }]
                  }]
                }]
              }]
            }"#
            .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.vulnerable_update_count, 0);
}

#[test]
fn single_apply_command_does_not_count_vulnerable_targets_when_vulnerabilities_are_hidden() {
    let session = session_with_vulnerability_visibility(false);

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("single-apply-command-does-not-count-vulnerable-targets-when-vulnerabilities-are-hidden.json"),
            workspace_root: None,
        },
        Some("update"),
        Some("left-pad"),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "1.1.0" },
              "vulns": [{
                "id": "OSV-1",
                "summary": "target issue",
                "affected": [{
                  "package": { "name": "left-pad" },
                  "ranges": [{
                    "events": [{ "introduced": "1.1.0" }, { "fixed": "2.0.0" }]
                  }]
                }]
              }]
            }"#
            .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.vulnerable_update_count, 0);
}

#[test]
fn apply_command_counts_authorization_required_failures() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-counts-authorization-required-failures.json"),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[RegistryResponseInput {
            package: "private-package".to_owned(),
            ecosystem: Npm,
            body: r#"{"status":401}"#.to_owned(),
        }],
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
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-does-not-count-forbidden-registry-failures-as-authorization-required.json"),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[RegistryResponseInput {
            package: "private-package".to_owned(),
            ecosystem: Npm,
            body: r#"{"status":403}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 0);
    assert_eq!(output.authorization_required_count, 0);
    assert_eq!(output.vulnerable_update_count, 0);
}
