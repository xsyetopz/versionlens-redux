#[test]
fn apply_command_updates_julia_compat_dependency() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///Project.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("apply-command-updates-julia-compat-dependency.toml"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("Example"),
        selected_version: Some("0.6.0"),
        responses: &[RegistryResponseInput {
            package: "Example".to_owned(),
            ecosystem: Julia,
            body: r#"[0.5.4]
git-tree-sha1 = "c5e5"

[0.6.0]
git-tree-sha1 = "d6f6"
"#
            .to_owned(),
        }],
    });
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "0.6.0");
}

#[test]
fn apply_command_updates_r_description_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///DESCRIPTION".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-r-description-dependency-preserving-operatorDESCRIPTION",
            ),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("dplyr"),
        selected_version: Some("1.1.4"),
        responses: &[RegistryResponseInput {
            package: "dplyr".to_owned(),
            ecosystem: Cran,
            body: "Package: dplyr\nVersion: 1.1.4\n".to_owned(),
        }],
    });
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, ">= 1.1.4");
}

#[test]
fn apply_command_updates_paket_dependencies_nuget_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///paket.dependencies".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-paket-dependencies-nuget-version.dependencies",
            ),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("Newtonsoft.Json"),
        selected_version: Some("13.0.3"),
        responses: &[RegistryResponseInput {
            package: "Newtonsoft.Json".to_owned(),
            ecosystem: Dotnet,
            body: r#"{"versions":["13.0.1","13.0.3"]}"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "paket.dependencies");
    assert_eq!(output.suggestions[0].dependency.requirement, "13.0.1");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "13.0.3");
}

#[test]
fn apply_command_does_not_update_paket_references_without_versions() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///paket.references".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture(
                "apply-command-does-not-update-paket-references-without-versions.references",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("Newtonsoft.Json"),
        &[RegistryResponseInput {
            package: "Newtonsoft.Json".to_owned(),
            ecosystem: Dotnet,
            body: r#"{"versions":["13.0.3"]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_update_dockerfile_digest_pinned_image() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Dockerfile".to_owned(),
            language_id: "dockerfile".to_owned(),
            text: package_file_fixture(
                "apply-command-does-not-update-dockerfile-digest-pinned-imageDockerfile",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("ubuntu"),
        &[RegistryResponseInput {
            package: "ubuntu".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"24.04","tag_status":"active","digest":"sha256-new"}]}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_update_compose_digest_pinned_image() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///compose.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture(
                "apply-command-does-not-update-compose-digest-pinned-image.yaml",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("ubuntu"),
        &[RegistryResponseInput {
            package: "ubuntu".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"24.04","tag_status":"active","digest":"sha256-new"}]}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_vcpkg_version_constraint() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///vcpkg.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-vcpkg-version-constraint.json"),
            workspace_root: None,
        },
        Some("update"),
        Some("fmt"),
        &[RegistryResponseInput {
            package: "fmt".to_owned(),
            ecosystem: Vcpkg,
            body: r#"{"versions":[{"version":"11.1.4"},{"version":"10.1.1#1"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "11.1.4");
}

#[test]
fn apply_command_does_not_update_vcpkg_baseline_dependency_without_version_constraint() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///vcpkg.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-does-not-update-vcpkg-baseline-dependency-without-version-constraint.json"),
            workspace_root: None,
        },
        Some("update"),
        Some("zlib"),
        &[RegistryResponseInput {
            package: "zlib".to_owned(),
            ecosystem: Vcpkg,
            body: r#"{"versions":[{"version":"1.3.1"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_swift_package_github_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Package.swift".to_owned(),
            language_id: "swift".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-swift-package-github-dependency.swift",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("swift-nio"),
        &[RegistryResponseInput {
            package: "apple/swift-nio".to_owned(),
            ecosystem: Swift,
            body: r#"[{"name":"2.66.0"},{"name":"2.65.0"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.66.0");
}
