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
        command: Some("update"),
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
