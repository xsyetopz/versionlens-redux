use std::fs::read_to_string;
use std::path::PathBuf;

use super::{
    ApplyCommandRequest, DocumentInput, RegistryResponseInput,
    session_with_vulnerability_visibility, standard_session,
};
use versionlens_parsers::Ecosystem::{
    AnsibleGalaxy, Bazel, CocoaPods, Docker, Helm, Nix, Npm, Terraform, Unity,
};

#[test]
fn apply_command_updates_only_selected_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-only-selected-dependency.json"),
            workspace_root: None,
        },
        None,
        Some("left-pad"),
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "is-odd".to_owned(),
                ecosystem: Npm,
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

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-selected-build-version.json"),
            workspace_root: None,
        },
        command: None,
        dependency_name: Some("left-pad"),
        selected_version: Some("1.0.0+build.3"),
        responses: &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
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
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.0.0+build.3");
}

#[test]
fn apply_command_updates_terraform_provider_version_without_replacing_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///main.tf".to_owned(),
            language_id: "terraform".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-terraform-provider-version-without-replacing-operator.tf",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("hashicorp/aws"),
        &[RegistryResponseInput {
            package: "hashicorp/aws".to_owned(),
            ecosystem: Terraform,
            body: r#"{"versions":[{"version":"6.0.0"},{"version":"6.1.0-beta.1"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "6.0.0");
}

#[test]
fn apply_command_updates_helm_chart_dependency_version_without_replacing_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Chart.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("apply-command-updates-helm-chart-dependency-version-without-replacing-operator.yaml"),
            workspace_root: None,
        },
        Some("update"),
        Some("mysql"),
        &[RegistryResponseInput {
            package: "mysql".to_owned(),
            ecosystem: Helm,
            body: "apiVersion: v1\nentries:\n  mysql:\n    - version: 4.0.0\n".to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "4.0.0");
}

#[test]
fn apply_command_updates_ansible_collection_requirement_without_replacing_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/requirements.yml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("apply-command-updates-ansible-collection-requirement-without-replacing-operator.yml"),
            workspace_root: None,
        },
        Some("update"),
        Some("community.general"),
        &[RegistryResponseInput {
            package: "community.general".to_owned(),
            ecosystem: AnsibleGalaxy,
            body: r#"{"data":[{"version":"8.0.0"},{"version":"7.5.0"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "8.0.0");
}

#[test]
fn apply_command_updates_bazel_module_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/MODULE.bazel".to_owned(),
            language_id: "starlark".to_owned(),
            text: package_file_fixture("apply-command-updates-bazel-module-dependencyMODULE.bazel"),
            workspace_root: None,
        },
        Some("update"),
        Some("rules_cc"),
        &[RegistryResponseInput {
            package: "rules_cc".to_owned(),
            ecosystem: Bazel,
            body: r#"{"versions":["0.0.9","0.0.10"]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "0.0.10");
}

#[test]
fn apply_command_updates_cocoapods_podfile_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/Podfile".to_owned(),
            language_id: "ruby".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-cocoapods-podfile-dependency-preserving-operatorPodfile",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("AFNetworking"),
        &[RegistryResponseInput {
            package: "AFNetworking".to_owned(),
            ecosystem: CocoaPods,
            body: r#"{"versions":[{"name":"5.0.0"},{"name":"4.0.1"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "~> 5.0.0");
}

#[test]
fn apply_command_updates_unity_project_manifest_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/Packages/manifest.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-unity-project-manifest-dependency.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("com.unity.timeline"),
        &[RegistryResponseInput {
            package: "com.unity.timeline".to_owned(),
            ecosystem: Unity,
            body: r#"{"dist-tags":{"latest":"1.8.7"},"versions":{"1.8.6":{},"1.8.7":{}}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.8.7");
}

#[test]
fn apply_command_updates_kustomization_image_new_tag() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/kustomization.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("apply-command-updates-kustomization-image-new-tag.yaml"),
            workspace_root: None,
        },
        Some("update"),
        Some("platform/nginx"),
        &[RegistryResponseInput {
            package: "platform/nginx".to_owned(),
            ecosystem: Docker,
            body: r#"{"tags":["1.26.0","1.25.3"]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.26.0");
}

#[test]
fn apply_command_updates_nix_flake_github_input_ref() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/flake.nix".to_owned(),
            language_id: "nix".to_owned(),
            text: package_file_fixture("apply-command-updates-nix-flake-github-input-ref.nix"),
            workspace_root: None,
        },
        Some("update"),
        Some("NixOS/nixpkgs"),
        &[RegistryResponseInput {
            package: "NixOS/nixpkgs".to_owned(),
            ecosystem: Nix,
            body: r#"[{"name":"nixos-24.05"},{"name":"nixos-23.11"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "24.05");
}

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

#[test]
fn apply_command_uses_code_lens_selector_for_duplicate_names() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "apply-command-uses-code-lens-selector-for-duplicate-names.json",
        ),
        workspace_root: None,
    };

    let responses = [RegistryResponseInput {
        package: "left-pad".to_owned(),
        ecosystem: Npm,
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
            text: package_file_fixture("apply-command-updates-only-requested-level.json"),
            workspace_root: None,
        },
        Some("updateMinor"),
        None,
        &[
            RegistryResponseInput {
                package: "major".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "minor".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "patch".to_owned(),
                ecosystem: Npm,
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
            text: package_file_fixture("apply-command-updates-ranged-dependency-to-requested-minor-choice.json"),
            workspace_root: None,
        },
        Some("updateMinor"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
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
            text: package_file_fixture("apply-command-updates-ranged-dependency-to-requested-patch-choice.json"),
            workspace_root: None,
        },
        Some("updatePatch"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
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
            text: package_file_fixture(
                "apply-command-level-filter-does-not-bump-project-version.json",
            ),
            workspace_root: None,
        },
        Some("updateMajor"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
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
            text: package_file_fixture(
                "apply-command-bulk-update-skips-project-version-edits.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn bulk_update_skips_prerelease_only_invalid_range_updates() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "bulk-update-skips-prerelease-only-invalid-range-updates.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
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

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/commands/update")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session command update fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}

include!("update_more.rs");
include!("update_more2.rs");
include!("update_more3.rs");
