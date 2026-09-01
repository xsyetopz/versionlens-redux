#[test]
fn pub_path_dependencies_resolve_as_directories() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/app/pubspec.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("pub-path-dependencies-resolve-as-directories.yaml"), None),
        &[RegistryResponseInput::new("http_parser".to_owned(), Pub, r#"{"latest":{"version":"9.9.9"}}"#.to_owned())],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "directory", Some("../../"));
}

#[test]
fn pub_sdk_dependencies_resolve_as_fixed_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/app/pubspec.yaml".to_owned(), "yaml".to_owned(), package_file_fixture(
                "pub-sdk-dependencies-resolve-as-fixed-without-registry-lookup.yaml",
            ), None),
        &[RegistryResponseInput::new("flutter".to_owned(), Pub, r#"{"latest":{"version":"9.9.9"}}"#.to_owned())],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "fixed", Some("sdk:flutter"));
}

#[test]
fn pub_workspace_paths_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("pub-workspace-directory");
    let packages = root.join("packages");
    let shared = packages.join("shared");
    create_dir_all(&shared).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(file_uri(&root.join("pubspec.yaml")), "yaml".to_owned(), package_file_fixture("pub-workspace-paths-resolve-as-directories.txt"), None),
        &[RegistryResponseInput::new("packages/shared".to_owned(), Pub, r#"{"latest":{"version":"9.9.9"}}"#.to_owned())],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("packages/shared")
    );
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn dub_sdl_path_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("dub-directory");
    create_two_local_dependencies(&root);
    let output = resolve_local_fixture(LocalFixtureCase {
        session: &session,
        root: &root,
        manifest: "dub.sdl",
        language_id: "plaintext",
        fixture_name: "dub-sdl-path-dependencies-resolve-as-directories.txt",
        package: "localdep",
        ecosystem: Dub,
        response: r#"{"versions":[{"version":"9.9.9"}]}"#,
    });

    assert_two_directory_suggestions(&output, "./localdep", "vendor/localdep");
    remove_dir_all(root).unwrap();
}

#[test]
fn dub_json_path_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("dub-json-directory");
    let local = root.join("localdep");
    create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(file_uri(&root.join("dub.json")), "json".to_owned(), package_file_fixture("dub-json-path-dependencies-resolve-as-directories.txt"), None),
        &[RegistryResponseInput::new("localdep".to_owned(), Dub, r#"{"versions":[{"version":"9.9.9"}]}"#.to_owned())],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "directory", Some("./localdep"));
    remove_dir_all(root).unwrap();
}

#[test]
fn gleam_path_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("gleam-directory");
    create_two_local_dependencies(&root);
    let output = resolve_local_fixture(LocalFixtureCase {
        session: &session,
        root: &root,
        manifest: "gleam.toml",
        language_id: "toml",
        fixture_name: "gleam-path-dependencies-resolve-as-directories.txt",
        package: "localdep",
        ecosystem: Hex,
        response: r#"{"releases":[{"version":"9.9.9"}]}"#,
    });

    assert_two_directory_suggestions(&output, "./localdep", "vendor/localdep");
    remove_dir_all(root).unwrap();
}

#[test]
fn gleam_git_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();
    let output = crate::support::tests::resolve_fixture_with_response(crate::support::tests::FixtureResolutionCase { session: &session, uri: "file:///repo/gleam.toml", language_id: "toml", fixture_name: "gleam-git-dependencies-are-fixed-without-registry-updates.toml", package: "my_library", ecosystem: Hex, response: r#"{"releases":[{"version":"9.9.9"}]}"# });

    crate::support::tests::assert_fixed_git_repository(&output);
}

#[test]
fn mix_umbrella_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();
    let output = crate::support::tests::resolve_fixture_with_response(crate::support::tests::FixtureResolutionCase { session: &session, uri: "file:///repo/mix.exs", language_id: "elixir", fixture_name: "mix-umbrella-dependencies-are-fixed-without-registry-updates.exs", package: "shared_app", ecosystem: Hex, response: r#"{"releases":[{"version":"9.9.9"}]}"# });

    assert_fixed_without_edits(&output, "umbrella dependency");
}

#[test]
fn rebar_git_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();
    let output = crate::support::tests::resolve_fixture_with_response(crate::support::tests::FixtureResolutionCase { session: &session, uri: "file:///repo/rebar.config", language_id: "erlang", fixture_name: "rebar-git-dependencies-are-fixed-without-registry-updates.config", package: "gettext", ecosystem: Hex, response: r#"{"releases":[{"version":"9.9.9"}]}"# });

    crate::support::tests::assert_fixed_git_repository(&output);
}

#[test]
fn rebar_mercurial_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();
    let output = crate::support::tests::resolve_fixture_with_response(crate::support::tests::FixtureResolutionCase { session: &session, uri: "file:///repo/rebar.config", language_id: "erlang", fixture_name: "rebar-mercurial-dependencies-are-fixed-without-registry-updates.config", package: "legacy", ecosystem: Hex, response: r#"{"releases":[{"version":"9.9.9"}]}"# });

    assert_fixed_without_edits(&output, "hg repository");
}

fn assert_fixed_without_edits(
    output: &crate::contract::ResolveDocumentOutput,
    latest: &str,
) {
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some(latest));
    assert!(output.edits.is_empty());
}

fn assert_two_directory_suggestions(
    output: &crate::contract::ResolveDocumentOutput,
    first: &str,
    second: &str,
) {
    assert_eq!(output.suggestions.len(), 2);
    for (suggestion, expected) in output.suggestions.iter().zip([first, second]) {
        assert_eq!(suggestion.status, "directory");
        assert_eq!(suggestion.latest.as_deref(), Some(expected));
    }
    assert!(output.edits.is_empty());
}

fn create_two_local_dependencies(root: &std::path::Path) {
    create_dir_all(root.join("localdep")).unwrap();
    create_dir_all(root.join("vendor/localdep")).unwrap();
}

struct LocalFixtureCase<'a> {
    session: &'a crate::VersionLensSession,
    root: &'a std::path::Path,
    manifest: &'a str,
    language_id: &'a str,
    fixture_name: &'a str,
    package: &'a str,
    ecosystem: versionlens_model::Ecosystem,
    response: &'a str,
}

fn resolve_local_fixture(case: LocalFixtureCase<'_>) -> crate::contract::ResolveDocumentOutput {
    case.session.resolve_document_with_responses(
        DocumentInput::new(
            file_uri(&case.root.join(case.manifest)),
            case.language_id.to_owned(),
            package_file_fixture(case.fixture_name),
            None,
        ),
        &[RegistryResponseInput::new(
            case.package.to_owned(),
            case.ecosystem,
            case.response.to_owned(),
        )],
    )
}

#[test]
fn python_direct_url_requirements_remain_fixed_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/requirements.txt".to_owned(), "pip-requirements".to_owned(), package_file_fixture(
                "python-direct-url-requirements-resolve-as-blank-versions-like-upstream.txt",
            ), None),
        &[RegistryResponseInput::new("local".to_owned(), Python, r#"{"info":{"version":"9.9.9"}}"#.to_owned())],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("https://example.test/local.whl#sha256=abc")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_workspace_and_catalog_dependencies_are_surfaced_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("workspace-and-catalog-dependencies-are-skipped.json"), None),
        &[],
    );

    assert_eq!(output.suggestions.len(), 2);
    crate::support::tests::assert_all_fixed_without_edits(&output);
}

#[test]
fn cargo_rust_version_is_terminal_and_not_updated() {
    let session = standard_session();
    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/Cargo.toml".to_owned(),
            "toml".to_owned(),
            "[package]\nname = \"demo\"\nrust-version = \"1.78\"\n".to_owned(),
            None,
        ),
        &[RegistryResponseInput::new(
            "rust".to_owned(),
            Cargo,
            r#"{"versions":["99.0.0"]}"#.to_owned(),
        )],
    );

    assert!(output
        .suggestions
        .iter()
        .any(|suggestion| suggestion.dependency.name == "rust"
            && suggestion.status == "fixed"));
    assert!(output.edits.is_empty());
}

#[test]
fn bun_trusted_dependency_name_arrays_are_fixed_by_default() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture(
                "bun-trusted-dependency-name-arrays-are-fixed-by-default.json",
            ), None),
        &[RegistryResponseInput::new("my-trusted-package".to_owned(), Npm, r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned())],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("trusted dependency")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_bundle_name_arrays_are_fixed_by_default() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("bundle-name-arrays-are-fixed-by-default.json"), None),
        &[
            RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned()),
            RegistryResponseInput::new("right-pad".to_owned(), Npm, r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned()),
        ],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("bundled dependency")
    );
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("bundled dependency")
    );
    assert!(output.edits.is_empty());
}
