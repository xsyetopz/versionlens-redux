use versionlens_parsers::Ecosystem::{Composer, Dub, Hex, Pub, Python};
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
            text: package_file_fixture("pub-sdk-dependencies-resolve-as-fixed-without-registry-lookup.yaml"),
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
            text: package_file_fixture("gleam-git-dependencies-are-fixed-without-registry-updates.toml"),
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
            text: package_file_fixture("mix-umbrella-dependencies-are-fixed-without-registry-updates.exs"),
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
            text: package_file_fixture("rebar-git-dependencies-are-fixed-without-registry-updates.config"),
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
            text: package_file_fixture("rebar-mercurial-dependencies-are-fixed-without-registry-updates.config"),
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
fn python_direct_url_requirements_resolve_as_blank_versions_like_upstream() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: package_file_fixture("python-direct-url-requirements-resolve-as-blank-versions-like-upstream.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Python,
            body: r#"{"info":{"version":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("9.9.9"));
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "==9.9.9");
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
            text: package_file_fixture("bun-trusted-dependency-name-arrays-are-fixed-by-default.json"),
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

#[test]
fn composer_platform_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("composer-platform-dependencies-are-fixed.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "phpunit/phpunit".to_owned(),
            ecosystem: Composer,
            body: r#"{"packages":{"phpunit/phpunit":[{"version":"10.5.0"}]}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("^8.3"));
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(output.suggestions[1].latest.as_deref(), Some("*"));
    assert_eq!(output.suggestions[2].latest.as_deref(), Some("10.5.0"));
}

#[test]
fn composer_stability_flags_allow_prerelease_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("composer-stability-flags-allow-prerelease-updates.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "acme/pkg".to_owned(),
            ecosystem: Composer,
            body: r#"{"packages":{"acme/pkg":[{"version":"1.0.0"},{"version":"1.1.0-beta.1"}]}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("1.1.0-beta.1")
    );
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "^1.1.0-beta.1@beta");
}

#[test]
fn fixed_composer_release_resolves_fixed_with_release_update_choices() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("fixed-composer-release-resolves-fixed-with-release-update-choices.json"),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Composer,
            body: r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v1.1.0" },
                  { "version": "v1.1.1" },
                  { "version": "v1.1.2" },
                  { "version": "v1.2.0" },
                  { "version": "v1.2.2" },
                  { "version": "v2.0.0" },
                  { "version": "v2.2.2" }
                ]
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("1.1.1"));
    assert!(output.edits.is_empty());
    assert_eq!(
        titles,
        [
            "🟡 fixed 1.1.1",
            "↑  latest 2.2.2",
            "↑  minor 1.2.2",
            "↑  patch 1.1.2"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "2.2.2"],
            vec!["updateMinor", "1.2.2"],
            vec!["updatePatch", "1.1.2"]
        ]
    );
}

#[test]
fn missing_fixed_composer_registry_version_resolves_no_match_with_update_choices() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("missing-fixed-composer-registry-version-resolves-no-match-with-update-choices.json"),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Composer,
            body: r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v0.5.1" },
                  { "version": "v0.6.0" },
                  { "version": "v1.0.0" }
                ]
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());
    assert_eq!(
        titles,
        [
            "⚪ no match",
            "↑  latest 1.0.0",
            "↑  minor 0.6.0",
            "↑  patch 0.5.1"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "1.0.0"],
            vec!["updateMinor", "0.6.0"],
            vec!["updatePatch", "0.5.1"]
        ]
    );
}

#[test]
fn invalid_composer_requirement_resolves_invalid_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("invalid-composer-requirement-resolves-invalid-without-registry-lookup.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Composer,
            body: r#"{"packages":{"php-parallel-lint/php-parallel-lint":[{"version":"v9.9.9"}]}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "invalid");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("invalid version")
    );
    assert!(output.edits.is_empty());
}
