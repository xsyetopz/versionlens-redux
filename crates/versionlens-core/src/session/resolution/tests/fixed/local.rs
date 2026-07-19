#[test]
fn cargo_path_dependencies_resolve_existing_relative_directories() {
    let session = standard_session();
    let root = local_test_root("cargo-path-directory");
    let cache = root.join("crates/versionlens-cache");
    create_dir_all(&cache).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&root.join("Cargo.toml")),
            language_id: "toml".to_owned(),
            text: r#"
[dependencies]
versionlens-cache = { path = "crates/versionlens-cache" }
versionlens-core = { version = "0.1.0", path = "crates/versionlens-cache" }
"#
            .to_owned(),
            workspace_root: Some(root.to_string_lossy().into_owned()),
        },
        &[RegistryResponseInput {
            package: "versionlens-cache".to_owned(),
            ecosystem: versionlens_model::Ecosystem::Cargo,
            body: r#"{"versions":[{"num":"9.9.9","yanked":false}]}"#.to_owned(),
        }],
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
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("missing-local-dependencies-return-directory-not-found-without-registry-lookup.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directoryNotFound");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("../local"));
    assert!(output.edits.is_empty());
}

#[test]
fn ruby_path_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("ruby-directory");
    let app = root.join("app");
    let local = app.join("vendor/local");
    create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&app.join("Gemfile")),
            language_id: "ruby".to_owned(),
            text: package_file_fixture("ruby-path-dependencies-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Ruby,
            body: r#"[{"number":"9.9.9"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("vendor/local")
    );
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn stack_custom_resolver_resolves_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///work/stack.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture(
                "stack-custom-resolver-resolves-as-fixed-without-registry-updates.yaml",
            ),
            workspace_root: None,
        },
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
        DocumentInput {
            uri: "file:///main.tf".to_owned(),
            language_id: "terraform".to_owned(),
            text: package_file_fixture(
                "terraform-builtin-provider-resolves-as-fixed-without-registry-updates.tf",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "terraform.io/builtin/terraform".to_owned(),
            ecosystem: Terraform,
            body: r#"{"versions":[{"version":"9.9.9"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("built-in provider")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn helm_local_and_repository_alias_dependencies_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Chart.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("helm-local-and-repository-alias-dependencies-resolve-as-fixed-without-registry-updates.yaml"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Helm,
            body: "apiVersion: v1\nentries:\n  local:\n    - version: 9.9.9\n".to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("local chart"));
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
        DocumentInput {
            uri: "file:///work/requirements.yml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture(
                "ansible-git-role-dependencies-resolve-as-fixed-without-registry-updates.yml",
            ),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn bazel_non_registry_overrides_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///work/MODULE.bazel".to_owned(),
            language_id: "starlark".to_owned(),
            text: package_file_fixture("bazel-non-registry-overrides-resolve-as-fixed-without-registry-updatesMODULE.bazel"),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
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
        DocumentInput {
            uri: "file:///work/flake.nix".to_owned(),
            language_id: "nix".to_owned(),
            text: package_file_fixture(
                "nix-local-inputs-resolve-as-fixed-without-registry-updates.nix",
            ),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("local flake"));
    assert!(output.edits.is_empty());
}

#[test]
fn renv_non_repository_packages_resolve_as_fixed_sources() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/app/renv.lock".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "renv-non-repository-packages-resolve-as-fixed-sources.lock",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "localpkg".to_owned(),
            ecosystem: Cran,
            body: "Package: localpkg\nVersion: 9.9.9\n".to_owned(),
        }],
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
        DocumentInput {
            uri: "file:///repo/DESCRIPTION".to_owned(),
            language_id: "r".to_owned(),
            text: "Package: example\nVersion: 0.1.0\nImports: dplyr (1.1.3)\n".to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "dplyr".to_owned(),
            ecosystem: Cran,
            body: "Package: dplyr\nVersion: 1.1.4\n\nPackage: unrelated\nVersion: 1.1.3\n"
                .to_owned(),
        }],
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
        DocumentInput {
            uri: file_uri(&app.join("go.mod")),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture("go-replace-local-dependencies-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "example.test/local".to_owned(),
            ecosystem: Go,
            body: "v9.9.9\n".to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("../local"));
    assert!(output.edits.is_empty());
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
        DocumentInput {
            uri: file_uri(&root.join("go.work")),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture("go-work-use-directories-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "./app".to_owned(),
            ecosystem: Go,
            body: "v9.9.9\n".to_owned(),
        }],
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
