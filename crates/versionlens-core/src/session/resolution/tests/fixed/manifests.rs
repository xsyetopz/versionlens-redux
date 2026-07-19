use versionlens_model::Ecosystem::{Composer, Dub, Hex, Pub, Python};
#[test]
fn pub_path_dependencies_resolve_as_directories() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/app/pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pub-path-dependencies-resolve-as-directories.yaml"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "http_parser".to_owned(),
            ecosystem: Pub,
            body: r#"{"latest":{"version":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("../../"));
    assert!(output.edits.is_empty());
}

#[test]
fn pub_sdk_dependencies_resolve_as_fixed_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/app/pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture(
                "pub-sdk-dependencies-resolve-as-fixed-without-registry-lookup.yaml",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "flutter".to_owned(),
            ecosystem: Pub,
            body: r#"{"latest":{"version":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("sdk:flutter"));
    assert!(output.edits.is_empty());
}

#[test]
fn pub_workspace_paths_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("pub-workspace-directory");
    let packages = root.join("packages");
    let shared = packages.join("shared");
    create_dir_all(&shared).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&root.join("pubspec.yaml")),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pub-workspace-paths-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "packages/shared".to_owned(),
            ecosystem: Pub,
            body: r#"{"latest":{"version":"9.9.9"}}"#.to_owned(),
        }],
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
    let local = root.join("localdep");
    let bare_local = root.join("vendor/localdep");
    create_dir_all(&local).unwrap();
    create_dir_all(&bare_local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&root.join("dub.sdl")),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture("dub-sdl-path-dependencies-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "localdep".to_owned(),
            ecosystem: Dub,
            body: r#"{"versions":[{"version":"9.9.9"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("./localdep"));
    assert_eq!(output.suggestions[1].status, "directory");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("vendor/localdep")
    );
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn dub_json_path_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("dub-json-directory");
    let local = root.join("localdep");
    create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&root.join("dub.json")),
            language_id: "json".to_owned(),
            text: package_file_fixture("dub-json-path-dependencies-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "localdep".to_owned(),
            ecosystem: Dub,
            body: r#"{"versions":[{"version":"9.9.9"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("./localdep"));
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn gleam_path_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("gleam-directory");
    let local = root.join("localdep");
    let bare_local = root.join("vendor/localdep");
    create_dir_all(&local).unwrap();
    create_dir_all(&bare_local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&root.join("gleam.toml")),
            language_id: "toml".to_owned(),
            text: package_file_fixture("gleam-path-dependencies-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "localdep".to_owned(),
            ecosystem: Hex,
            body: r#"{"releases":[{"version":"9.9.9"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("./localdep"));
    assert_eq!(output.suggestions[1].status, "directory");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("vendor/localdep")
    );
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn gleam_git_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/gleam.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture(
                "gleam-git-dependencies-are-fixed-without-registry-updates.toml",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "my_library".to_owned(),
            ecosystem: Hex,
            body: r#"{"releases":[{"version":"9.9.9"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn mix_umbrella_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/mix.exs".to_owned(),
            language_id: "elixir".to_owned(),
            text: package_file_fixture(
                "mix-umbrella-dependencies-are-fixed-without-registry-updates.exs",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "shared_app".to_owned(),
            ecosystem: Hex,
            body: r#"{"releases":[{"version":"9.9.9"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("umbrella dependency")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn rebar_git_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/rebar.config".to_owned(),
            language_id: "erlang".to_owned(),
            text: package_file_fixture(
                "rebar-git-dependencies-are-fixed-without-registry-updates.config",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "gettext".to_owned(),
            ecosystem: Hex,
            body: r#"{"releases":[{"version":"9.9.9"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn rebar_mercurial_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/rebar.config".to_owned(),
            language_id: "erlang".to_owned(),
            text: package_file_fixture(
                "rebar-mercurial-dependencies-are-fixed-without-registry-updates.config",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "legacy".to_owned(),
            ecosystem: Hex,
            body: r#"{"releases":[{"version":"9.9.9"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("hg repository")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn python_direct_url_requirements_remain_fixed_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: package_file_fixture(
                "python-direct-url-requirements-resolve-as-blank-versions-like-upstream.txt",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Python,
            body: r#"{"info":{"version":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("https://example.test/local.whl#sha256=abc")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_workspace_and_catalog_dependencies_are_skipped() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("npm-workspace-and-catalog-dependencies-are-skipped.json"),
            workspace_root: None,
        },
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn bun_trusted_dependency_name_arrays_are_fixed_by_default() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "bun-trusted-dependency-name-arrays-are-fixed-by-default.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "my-trusted-package".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        }],
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
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("npm-bundle-name-arrays-are-fixed-by-default.json"),
            workspace_root: None,
        },
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "right-pad".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
            },
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
