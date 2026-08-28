#[test]
fn cargo_path_dependencies_resolve_existing_relative_directories() {
    let session = standard_session();
    let root = local_test_root("cargo-path-directory");
    let cache = root.join("crates/versionlens-cache");
    create_dir_all(&cache).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(file_uri(&root.join("Cargo.toml")), "toml".to_owned(), r#"
[dependencies]
versionlens-cache = { path = "crates/versionlens-cache" }
versionlens-core = { version = "0.1.0", path = "crates/versionlens-cache" }
"#
            .to_owned(), Some(root.to_string_lossy().into_owned())),
        &[RegistryResponseInput::new("versionlens-cache".to_owned(), versionlens_model::Ecosystem::Cargo, r#"{"versions":[{"num":"9.9.9","yanked":false}]}"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("crates/versionlens-cache")
    );
    assert_eq!(output.suggestions[1].status, "directory");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("crates/versionlens-cache")
    );
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn missing_local_dependencies_return_directory_not_found_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/project/package.json".to_owned(), "json".to_owned(), package_file_fixture("missing-local-dependencies-return-directory-not-found-without-registry-lookup.json"), None),
        &[RegistryResponseInput::new("local".to_owned(), Npm, r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned())],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "directoryNotFound", Some("../local"));
}

#[test]
fn ruby_path_dependencies_resolve_as_directories() {
    assert_ruby_path_dependency_fixture(
        "ruby-path-dependencies-resolve-as-directories.txt",
        "local",
        "ruby-directory",
    );
}

#[test]
fn stack_custom_resolver_resolves_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///work/stack.yaml".to_owned(), "yaml".to_owned(), package_file_fixture(
                "stack-custom-resolver-resolves-as-fixed-without-registry-updates.yaml",
            ), None),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest,
        Some("stack resolver".to_owned())
    );
}

#[test]
fn terraform_builtin_provider_resolves_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///main.tf".to_owned(), "terraform".to_owned(), package_file_fixture(
                "terraform-builtin-provider-resolves-as-fixed-without-registry-updates.tf",
            ), None),
        &[RegistryResponseInput::new("terraform.io/builtin/terraform".to_owned(), Terraform, r#"{"versions":[{"version":"9.9.9"}]}"#.to_owned())],
    );

    crate::support::tests::assert_fixed_suggestion(&output, "built-in provider");
}

#[test]
fn helm_local_and_repository_alias_dependencies_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///Chart.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("helm-local-and-repository-alias-dependencies-resolve-as-fixed-without-registry-updates.yaml"), None),
        &[RegistryResponseInput::new("local".to_owned(), Helm, "apiVersion: v1\nentries:\n  local:\n    - version: 9.9.9\n".to_owned())],
    );

    assert_eq!(output.suggestions.len(), 2);
    crate::support::tests::assert_suggestion(&output, 0, "fixed", Some("local chart"));
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("repository alias")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn ansible_git_role_dependencies_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///work/requirements.yml".to_owned(), "yaml".to_owned(), package_file_fixture(
                "ansible-git-role-dependencies-resolve-as-fixed-without-registry-updates.yml",
            ), None),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    crate::support::tests::assert_fixed_git_repository(&output);
}

#[test]
fn bazel_non_registry_overrides_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///work/MODULE.bazel".to_owned(), "starlark".to_owned(), package_file_fixture("bazel-non-registry-overrides-resolve-as-fixed-without-registry-updatesMODULE.bazel"), None),
        &[],
    );

    assert_eq!(output.suggestions.len(), 2);
    crate::session::resolution::tests::assert_fixed_git_repository_suggestion(&output);
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("local module")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn nix_local_inputs_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///work/flake.nix".to_owned(), "nix".to_owned(), package_file_fixture(
                "nix-local-inputs-resolve-as-fixed-without-registry-updates.nix",
            ), None),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    crate::support::tests::assert_suggestion_without_edits(&output, 0, "fixed", Some("local flake"));
}

#[test]
fn renv_non_repository_packages_resolve_as_fixed_sources() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/app/renv.lock".to_owned(), "json".to_owned(), package_file_fixture(
                "renv-non-repository-packages-resolve-as-fixed-sources.lock",
            ), None),
        &[RegistryResponseInput::new("localpkg".to_owned(), Cran, "Package: localpkg\nVersion: 9.9.9\n".to_owned())],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("local package")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn cran_fixed_requirements_ignore_versions_from_other_packages() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/DESCRIPTION".to_owned(), "r".to_owned(), "Package: example\nVersion: 0.1.0\nImports: dplyr (1.1.3)\n".to_owned(), None),
        &[RegistryResponseInput::new("dplyr".to_owned(), Cran, "Package: dplyr\nVersion: 1.1.4\n\nPackage: unrelated\nVersion: 1.1.3\n"
                .to_owned())],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[1].dependency.name, "dplyr");
    assert_eq!(output.suggestions[1].status, "noMatch");
    assert!(output.edits.is_empty());
}

#[test]
fn go_replace_local_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("go-directory");
    let app = root.join("app");
    let local = root.join("local");
    create_dir_all(&app).unwrap();
    create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(file_uri(&app.join("go.mod")), "go.mod".to_owned(), package_file_fixture("go-replace-local-dependencies-resolve-as-directories.txt"), None),
        &[RegistryResponseInput::new("example.test/local".to_owned(), Go, "v9.9.9\n".to_owned())],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "directory", Some("../local"));
    remove_dir_all(root).unwrap();
}

#[test]
fn go_work_use_directories_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("go-work-use-directory");
    let app = root.join("app");
    let lib = root.join("lib");
    create_dir_all(&app).unwrap();
    create_dir_all(&lib).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(file_uri(&root.join("go.work")), "go.mod".to_owned(), package_file_fixture("go-work-use-directories-resolve-as-directories.txt"), None),
        &[RegistryResponseInput::new("./app".to_owned(), Go, "v9.9.9\n".to_owned())],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status == "directory")
    );
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("./app"));
    assert_eq!(output.suggestions[1].latest.as_deref(), Some("./lib"));
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}
