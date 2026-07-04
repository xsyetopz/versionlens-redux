use super::{
    DocumentInput, Ecosystem, RegistryResponseInput, session_with_vulnerability_visibility,
    standard_session,
};

#[test]
fn apply_command_updates_only_selected_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":"1.0.0","is-odd":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        None,
        Some("left-pad"),
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Ecosystem::Npm,
                body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "is-odd".to_owned(),
                ecosystem: Ecosystem::Npm,
                body: r#"{"dist-tags":{"latest":"3.0.0"}}"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "left-pad");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn apply_command_updates_selected_build_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":"1.0.0+build.1"}}"#.to_owned(),
            workspace_root: None,
        },
        None,
        Some("left-pad"),
        Some("1.0.0+build.3"),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{
              "dist-tags": { "latest": "1.0.0+build.2" },
              "versions": {
                "1.0.0+build.1": {},
                "1.0.0+build.2": {},
                "1.0.0+build.3": {}
              }
            }"#
            .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.0.0+build.3");
}

#[test]
fn apply_command_does_not_count_vulnerability_fixed_by_update() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":"1.0.0","is-odd":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Ecosystem::Npm,
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
                ecosystem: Ecosystem::Npm,
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
            text: r#"{"dependencies":{"left-pad":"1.0.0","is-odd":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("update"),
        Some("left-pad"),
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Ecosystem::Npm,
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
                ecosystem: Ecosystem::Npm,
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
            text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
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
            text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("update"),
        Some("left-pad"),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
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
            text: r#"{"dependencies":{"private-package":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[RegistryResponseInput {
            package: "private-package".to_owned(),
            ecosystem: Ecosystem::Npm,
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
            text: r#"{"dependencies":{"private-package":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[RegistryResponseInput {
            package: "private-package".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"status":403}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 0);
    assert_eq!(output.authorization_required_count, 0);
    assert_eq!(output.vulnerable_update_count, 0);
}

#[test]
fn apply_command_uses_code_lens_selector_for_duplicate_names() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"},"devDependencies":{"left-pad":"1.0.0"}}"#
            .to_owned(),
        workspace_root: None,
    };

    let responses = [RegistryResponseInput {
        package: "left-pad".to_owned(),
        ecosystem: Ecosystem::Npm,
        body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
    }];
    session.resolve_document_with_responses(input.clone(), &responses);
    let analyzed = session.analyze_document(input.clone());
    let selector = analyzed
        .code_lenses
        .iter()
        .find(|lens| lens.command == "versionlens.suggestion.onUpdateDependency")
        .and_then(|lens| lens.arguments.get(1))
        .expect("update code lens selector")
        .clone();
    let output = session.apply_command(input, None, Some(&selector), &responses);

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "dependencies");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn apply_command_updates_only_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"major":"1.0.0","minor":"1.0.0","patch":"1.0.0"}}"#
                .to_owned(),
            workspace_root: None,
        },
        Some("updateMinor"),
        None,
        &[
            RegistryResponseInput {
                package: "major".to_owned(),
                ecosystem: Ecosystem::Npm,
                body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "minor".to_owned(),
                ecosystem: Ecosystem::Npm,
                body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "patch".to_owned(),
                ecosystem: Ecosystem::Npm,
                body: r#"{"dist-tags":{"latest":"1.0.1"}}"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "minor");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn apply_command_updates_ranged_dependency_to_requested_minor_choice() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":"~1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("updateMinor"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"1.0.1":{},"1.1.0":{},"2.0.0":{}}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "left-pad");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "~1.1.0");
}

#[test]
fn apply_command_updates_ranged_dependency_to_requested_patch_choice() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":"<=1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("updatePatch"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"1.0.1":{},"1.1.0":{},"2.0.0":{}}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "left-pad");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "<=1.0.1");
}

#[test]
fn apply_command_level_filter_does_not_bump_project_version() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"version":"1.2.3","dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("updateMajor"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "left-pad");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

#[test]
fn apply_command_bulk_update_skips_project_version_edits() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"version":"1.2.3"}"#.to_owned(),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn bulk_update_skips_prerelease_only_invalid_range_updates() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":">1 <1"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{
              "dist-tags": { "latest": "5.0.0-beta.1" },
              "versions": {
                "5.0.0-beta.1": {}
              }
            }"#
            .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "invalidRange");
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_preserves_semver_requirement_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":"^1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("update"),
        Some("left-pad"),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "^2.0.0");
}

#[test]
fn apply_command_updates_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"version":"1.2.3","dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("updateMajor"),
        Some("1.2.3"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

#[test]
fn apply_command_updates_prerelease_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"version":"1.2.3-beta.4","dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("updateRelease"),
        Some("1.2.3-beta.4"),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.3");
}

#[test]
fn apply_command_updates_only_project_versions_for_prerelease_command() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"version":"1.2.3-beta.4","dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        Some("updatePrerelease"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.3-beta.5");
}

#[test]
fn apply_command_updates_cargo_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: "[package]\nname = \"demo\"\nversion = \"1.2.3\"\n".to_owned(),
            workspace_root: None,
        },
        Some("updatePatch"),
        Some("version"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "package");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn apply_command_updates_bare_requirements_with_equals_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: "importlib-metadata; python_version < '3.8'\n".to_owned(),
            workspace_root: None,
        },
        None,
        Some("importlib-metadata"),
        &[RegistryResponseInput {
            package: "importlib-metadata".to_owned(),
            ecosystem: Ecosystem::Python,
            body: r#"{"info":{"version":"8.7.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "==8.7.0");
}

#[test]
fn apply_command_updates_empty_pipfile_requirements_with_equals_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Pipfile".to_owned(),
            language_id: "toml".to_owned(),
            text: "[dev-packages]\nmagic = \"\"\n".to_owned(),
            workspace_root: None,
        },
        None,
        Some("magic"),
        &[RegistryResponseInput {
            package: "magic".to_owned(),
            ecosystem: Ecosystem::Python,
            body: r#"{"info":{"version":"1.2.3"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "==1.2.3");
}

#[test]
fn apply_command_ignores_dotnet_package_reference_child_version_for_upstream_parity() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///app.csproj".to_owned(),
            language_id: "xml".to_owned(),
            text: r#"<Project>
  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk">
      <Version>18.7.0</Version>
    </PackageReference>
  </ItemGroup>
</Project>"#
                .to_owned(),
            workspace_root: None,
        },
        Some("update"),
        Some("Microsoft.NET.Test.Sdk"),
        &[RegistryResponseInput {
            package: "Microsoft.NET.Test.Sdk".to_owned(),
            ecosystem: Ecosystem::Dotnet,
            body: r#"{"versions":["18.7.0","18.8.0"]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.requirement, "*");
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_inserts_missing_deno_import_versions() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: r#"{"imports":{"@std/assert":"jsr:@std/assert"}}"#.to_owned(),
            workspace_root: None,
        },
        None,
        Some("@std/assert"),
        &[RegistryResponseInput {
            package: "@std/assert".to_owned(),
            ecosystem: Ecosystem::Deno,
            body: r#"{"versions":{"1.0.1":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "jsr:@std/assert@1.0.1");
}

#[test]
fn apply_command_updates_deno_jsr_import_aliases_by_specifier_package() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: r#"{"imports":{"luca":"jsr:@luca/cases@1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        None,
        Some("luca"),
        &[RegistryResponseInput {
            package: "@luca/cases".to_owned(),
            ecosystem: Ecosystem::Deno,
            body: r#"{"versions":{"1.1.0":{},"1.0.0":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "jsr:@luca/cases@1.1.0");
}

#[test]
fn apply_command_ignores_pub_hosted_dependency_without_version_for_upstream_parity() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: "dependencies:\n  hosted_dep:\n    hosted:\n      name: hosted_alias\n      url: https://pub.example.test\n".to_owned(),
            workspace_root: None,
        },
        Some("update"),
        Some("hosted_dep"),
        &[RegistryResponseInput {
            package: "hosted_alias".to_owned(),
            ecosystem: Ecosystem::Pub,
            body: r#"{"latest":{"version":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}
