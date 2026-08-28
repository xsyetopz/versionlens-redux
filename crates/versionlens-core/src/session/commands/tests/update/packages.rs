#[test]
fn apply_command_updates_julia_compat_dependency() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///Project.toml".to_owned(), "toml".to_owned(), package_file_fixture("command-updates-julia-compat-dependency.toml"), None),
        command: Some("update"),
        dependency_name: Some("Example"),
        selected_version: Some("0.6.0"),
        responses: &[RegistryResponseInput::new("Example".to_owned(), Julia, r#"[0.5.4]
git-tree-sha1 = "c5e5"

[0.6.0]
git-tree-sha1 = "d6f6"
"#
            .to_owned())],
    });
    assert_single_edit(&output, "0.6.0");
}

#[test]
fn apply_command_updates_r_description_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///DESCRIPTION".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-updates-r-description-dependency-preserving-operatorDESCRIPTION",
            ), None),
        command: Some("update"),
        dependency_name: Some("dplyr"),
        selected_version: Some("1.1.4"),
        responses: &[RegistryResponseInput::new("dplyr".to_owned(), Cran, "Package: dplyr\nVersion: 1.1.4\n".to_owned())],
    });
    assert_single_edit(&output, ">= 1.1.4");
}

#[test]
fn apply_command_updates_paket_dependencies_nuget_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///paket.dependencies".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-updates-paket-dependencies-nuget-version.dependencies",
            ), None),
        command: Some("update"),
        dependency_name: Some("Newtonsoft.Json"),
        selected_version: Some("13.0.3"),
        responses: &[RegistryResponseInput::new("Newtonsoft.Json".to_owned(), Dotnet, r#"{"versions":["13.0.1","13.0.3"]}"#.to_owned())],
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
        DocumentInput::new("file:///paket.references".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-does-not-update-paket-references-without-versions.references",
            ), None),
        Some("update"),
        Some("Newtonsoft.Json"),
        &[RegistryResponseInput::new("Newtonsoft.Json".to_owned(), Dotnet, r#"{"versions":["13.0.3"]}"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_update_dockerfile_digest_pinned_image() {
    assert_digest_pinned_image_is_not_updated(
        "file:///Dockerfile",
        "dockerfile",
        "command-does-not-update-dockerfile-digest-pinned-imageDockerfile",
    );
}

#[test]
fn apply_command_does_not_update_compose_digest_pinned_image() {
    assert_digest_pinned_image_is_not_updated(
        "file:///compose.yaml",
        "yaml",
        "command-does-not-update-compose-digest-pinned-image.yaml",
    );
}

fn assert_digest_pinned_image_is_not_updated(uri: &str, language_id: &str, fixture: &str) {
    let output = standard_session().apply_command(
        DocumentInput::new(
            uri.to_owned(),
            language_id.to_owned(),
            package_file_fixture(fixture),
            None,
        ),
        Some("update"),
        Some("ubuntu"),
        &[RegistryResponseInput::new(
            "ubuntu".to_owned(),
            Docker,
            r#"{"results":[{"name":"24.04","tag_status":"active","digest":"sha256-new"}]}"#.to_owned(),
        )],
    );
    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_vcpkg_version_constraint() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///vcpkg.json".to_owned(), "json".to_owned(), package_file_fixture("command-updates-vcpkg-version-constraint.json"), None),
        Some("update"),
        Some("fmt"),
        &[RegistryResponseInput::new("fmt".to_owned(), Vcpkg, r#"{"versions":[{"version":"11.1.4"},{"version":"10.1.1#1"}]}"#.to_owned())],
    );

    assert_single_edit(&output, "11.1.4");
}

#[test]
fn apply_command_does_not_update_vcpkg_baseline_dependency_without_version_constraint() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///vcpkg.json".to_owned(), "json".to_owned(), package_file_fixture("command-does-not-update-vcpkg-unconstrained-vcpkg-dependency.json"), None),
        Some("update"),
        Some("zlib"),
        &[RegistryResponseInput::new("zlib".to_owned(), Vcpkg, r#"{"versions":[{"version":"1.3.1"}]}"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_swift_package_github_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///Package.swift".to_owned(), "swift".to_owned(), package_file_fixture(
                "command-updates-swift-package-github-dependency.swift",
            ), None),
        Some("update"),
        Some("swift-nio"),
        &[RegistryResponseInput::new("apple/swift-nio".to_owned(), Swift, r#"[{"name":"2.66.0"},{"name":"2.65.0"}]"#.to_owned())],
    );

    assert_single_edit(&output, "2.66.0");
}
