#[test]
fn apply_command_updates_selected_build_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("command-updates-selected-build-version.json"), None),
        command: Some("update"),
        dependency_name: Some("left-pad"),
        selected_version: Some("1.0.0+build.3"),
        responses: &[RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{
              "dist-tags": { "latest": "1.0.0+build.2" },
              "versions": {
                "1.0.0+build.1": {},
                "1.0.0+build.2": {},
                "1.0.0+build.3": {}
              }
            }"#
            .to_owned())],
    });

    assert_single_edit(&output, "1.0.0+build.3");
}

#[test]
fn apply_command_updates_terraform_provider_version_without_replacing_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///main.tf".to_owned(), "terraform".to_owned(), package_file_fixture(
                "command-updates-terraform-provider-version-without-replacing-operator.tf",
            ), None),
        Some("update"),
        Some("hashicorp/aws"),
        &[RegistryResponseInput::new("hashicorp/aws".to_owned(), Terraform, r#"{"versions":[{"version":"6.0.0"},{"version":"6.1.0-beta.1"}]}"#.to_owned())],
    );

    assert_single_edit(&output, "6.0.0");
}

#[test]
fn apply_command_updates_helm_chart_dependency_version_without_replacing_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///Chart.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("command-updates-helm-chart-dependency-version-without-replacing-operator.yaml"), None),
        Some("update"),
        Some("mysql"),
        &[RegistryResponseInput::new("mysql".to_owned(), Helm, "apiVersion: v1\nentries:\n  mysql:\n    - version: 4.0.0\n".to_owned())],
    );

    assert_single_edit(&output, "4.0.0");
}

#[test]
fn apply_command_updates_ansible_collection_requirement_without_replacing_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///work/requirements.yml".to_owned(), "yaml".to_owned(), package_file_fixture("command-updates-ansible-collection-requirement-without-replacing-operator.yml"), None),
        Some("update"),
        Some("community.general"),
        &[RegistryResponseInput::new("community.general".to_owned(), AnsibleGalaxy, r#"{"data":[{"version":"8.0.0"},{"version":"7.5.0"}]}"#.to_owned())],
    );

    assert_single_edit(&output, "8.0.0");
}

#[test]
fn apply_command_updates_bazel_module_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///work/MODULE.bazel".to_owned(), "starlark".to_owned(), package_file_fixture("command-updates-bazel-module-dependencyMODULE.bazel"), None),
        Some("update"),
        Some("rules_cc"),
        &[RegistryResponseInput::new("rules_cc".to_owned(), Bazel, r#"{"versions":["0.0.9","0.0.10"]}"#.to_owned())],
    );

    assert_single_edit(&output, "0.0.10");
}

#[test]
fn apply_command_updates_cocoapods_podfile_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///work/Podfile".to_owned(), "ruby".to_owned(), package_file_fixture(
                "command-updates-cocoapods-podfile-dependency-preserving-operatorPodfile",
            ), None),
        Some("update"),
        Some("AFNetworking"),
        &[RegistryResponseInput::new("AFNetworking".to_owned(), CocoaPods, r#"{"versions":[{"name":"5.0.0"},{"name":"4.0.1"}]}"#.to_owned())],
    );

    assert_single_edit(&output, "~> 5.0.0");
}

#[test]
fn apply_command_updates_unity_project_manifest_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///work/Packages/manifest.json".to_owned(), "json".to_owned(), package_file_fixture(
                "command-updates-unity-project-manifest-dependency.json",
            ), None),
        Some("update"),
        Some("com.unity.timeline"),
        &[RegistryResponseInput::new("com.unity.timeline".to_owned(), Unity, r#"{"dist-tags":{"latest":"1.8.7"},"versions":{"1.8.6":{},"1.8.7":{}}}"#
                .to_owned())],
    );

    assert_single_edit(&output, "1.8.7");
}

#[test]
fn apply_command_updates_kustomization_image_new_tag() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///work/kustomization.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("command-updates-kustomization-image-new-tag.yaml"), None),
        Some("update"),
        Some("platform/nginx"),
        &[RegistryResponseInput::new("platform/nginx".to_owned(), Docker, r#"{"tags":["1.26.0","1.25.3"]}"#.to_owned())],
    );

    assert_single_edit(&output, "1.26.0");
}

#[test]
fn apply_command_updates_nix_flake_github_input_ref() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///work/flake.nix".to_owned(), "nix".to_owned(), package_file_fixture("command-updates-nix-flake-github-input-ref.nix"), None),
        Some("update"),
        Some("NixOS/nixpkgs"),
        &[RegistryResponseInput::new("NixOS/nixpkgs".to_owned(), Nix, r#"[{"name":"nixos-24.05"},{"name":"nixos-23.11"}]"#.to_owned())],
    );

    assert_single_edit(&output, "24.05");
}
