use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    DocumentInput, Ecosystem, RegistryResponseInput, session_with_dependency_properties,
    standard_session,
};

mod dotnet;
mod npm;
mod registry_sources;

#[test]
fn missing_local_dependencies_return_directory_not_found_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"local":"file:../local"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Ecosystem::Npm,
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
    std::fs::create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&app.join("Gemfile")),
            language_id: "ruby".to_owned(),
            text: r#"gem "local", path: "vendor/local""#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Ecosystem::Ruby,
            body: r#"[{"number":"9.9.9"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("vendor/local")
    );
    assert!(output.edits.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn go_replace_local_dependencies_resolve_as_no_match_like_upstream() {
    let session = standard_session();
    let root = local_test_root("go-directory");
    let app = root.join("app");
    let local = root.join("local");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&app.join("go.mod")),
            language_id: "go.mod".to_owned(),
            text: "replace example.test/local => ../local\n".to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "example.test/local".to_owned(),
            ecosystem: Ecosystem::Go,
            body: "v9.9.9\n".to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("v9.9.9"));
    assert!(output.edits.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pub_path_dependencies_resolve_as_directories() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/app/pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: "dependencies:\n  http_parser:\n    path: ../../\n".to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "http_parser".to_owned(),
            ecosystem: Ecosystem::Pub,
            body: r#"{"latest":{"version":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("../../"));
    assert!(output.edits.is_empty());
}

#[test]
fn python_direct_url_requirements_resolve_as_blank_versions_like_upstream() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: "local @ https://example.test/local.whl#sha256=abc\n".to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Ecosystem::Python,
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
            text: r#"{"dependencies":{"workspace-only":"workspace:*","catalog-only":"catalog:"}}"#
                .to_owned(),
            workspace_root: None,
        },
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn npm_bundle_name_arrays_are_fixed() {
    let session = session_with_dependency_properties(
        Ecosystem::Npm,
        &["bundledDependencies", "bundleDependencies"],
    );

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"bundledDependencies":["left-pad"],"bundleDependencies":["right-pad"]}"#
                .to_owned(),
            workspace_root: None,
        },
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Ecosystem::Npm,
                body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "right-pad".to_owned(),
                ecosystem: Ecosystem::Npm,
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
            text: r#"{"require":{"php":"^8.3","ext-json":"*","phpunit/phpunit":"^10.0"}}"#
                .to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "phpunit/phpunit".to_owned(),
            ecosystem: Ecosystem::Composer,
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
fn fixed_composer_release_resolves_fixed_with_release_update_choices() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"require":{"php-parallel-lint/php-parallel-lint":"1.1.1"}}"#.to_owned(),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Ecosystem::Composer,
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
                .map(String::as_str)
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
        text: r#"{"require":{"php-parallel-lint/php-parallel-lint":"0.5.0"}}"#.to_owned(),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Ecosystem::Composer,
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
                .map(String::as_str)
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
            text:
                r#"{"require":{"php-parallel-lint/php-parallel-lint":"definitely-not-a-version"}}"#
                    .to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Ecosystem::Composer,
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

#[test]
fn git_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: r#"[dependencies]
remote = { git = "https://example.test/repo.git" }
"#
            .to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "remote".to_owned(),
            ecosystem: Ecosystem::Cargo,
            body: r#"{"crate":{"max_version":"9.9.9"}}"#.to_owned(),
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
fn cargo_registry_dependencies_use_workspace_cargo_config_urls() {
    let root =
        std::env::temp_dir().join(format!("versionlens-cargo-registry-{}", std::process::id()));
    std::fs::create_dir_all(root.join(".cargo")).unwrap();
    std::fs::write(
        root.join(".cargo/config.toml"),
        "[registries.private]\nindex = 'https://cargo.example.test/api/'\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("Cargo.toml").display()),
        language_id: "toml".to_owned(),
        text: r#"
[dependencies]
private = { version = "1.0", registry = "private" }
"#
        .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "private");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://cargo.example.test/api/private/versions"]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cargo_crates_io_source_replacement_uses_workspace_cargo_config_url() {
    let root =
        std::env::temp_dir().join(format!("versionlens-cargo-replace-{}", std::process::id()));
    std::fs::create_dir_all(root.join(".cargo")).unwrap();
    std::fs::write(
        root.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = 'mirror'\n[source.mirror]\nregistry = 'sparse+https://mirror.example.test/api/'\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("Cargo.toml").display()),
        language_id: "toml".to_owned(),
        text: "[dependencies]\nserde = \"1.0\"\n".to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "serde");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://mirror.example.test/api/serde/versions"]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_dependencies_use_workspace_bunfig_registry_urls() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-bunfig-registry-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("bunfig.toml"),
        "[install]\nregistry = 'https://registry.example.test/npm'\n[install.scopes]\n'@scope' = { url = 'https://scope.example.test/npm', token = 'secret' }\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0","@scope/pkg":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.example.test/npm/left-pad"]
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec!["https://scope.example.test/npm/@scope%2fpkg"]
    );
    assert_eq!(
        context.auth_headers_for_url(
            Ecosystem::Npm,
            "https://scope.example.test/npm/@scope%2fpkg"
        )[0]
        .value,
        "Bearer secret"
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn composer_repository_filters_route_matching_packages_only() {
    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{
  "repositories": [
    {
      "type": "composer",
      "url": "https://private.packages.example.test",
      "only": ["acme/*"],
      "exclude": ["acme/blocked"]
    }
  ],
  "require": {
    "acme/private": "1.0.0",
    "acme/blocked": "1.0.0",
    "vendor/public": "1.0.0"
  }
}"#
        .to_owned(),
        workspace_root: None,
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "acme/private");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://private.packages.example.test/acme/private.json"]
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec!["https://repo.packagist.org/p2/acme/blocked.json"]
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[2], &context),
        vec!["https://repo.packagist.org/p2/vendor/public.json"]
    );
}

#[test]
fn composer_can_disable_default_packagist_registry() {
    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{
  "repositories": {
    "packagist.org": false,
    "private": {
      "type": "composer",
      "url": "https://private.packages.example.test",
      "only": ["acme/*"]
    }
  },
  "require": {
    "acme/private": "1.0.0",
    "vendor/public": "1.0.0"
  }
}"#
        .to_owned(),
        workspace_root: None,
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://private.packages.example.test/acme/private.json"]
    );
    assert!(
        session
            .registry_urls_with_context(&dependencies[1], &context)
            .is_empty()
    );
}

#[test]
fn explicit_docker_registries_return_no_match_from_mcr_shaped_responses() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///compose.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: "services:\n  app:\n    image: registry.example.test/team/app:1.0.0\n".to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "team/app".to_owned(),
            ecosystem: Ecosystem::Docker,
            body: r#"{"results":[{"name":"2.0.0","images":[{"digest":"sha256:abc"}]}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());
}

#[test]
fn docker_compose_bare_build_contexts_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("docker-directory");
    let local = root.join("backend/dockerfile");
    std::fs::create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&root.join("compose.yaml")),
            language_id: "yaml".to_owned(),
            text: "services:\n  app:\n    build: backend\n".to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "backend/dockerfile".to_owned(),
            ecosystem: Ecosystem::Docker,
            body: r#"{"results":[{"name":"2.0.0","images":[{"digest":"sha256:abc"}]}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("backend/dockerfile")
    );
    assert!(output.edits.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_git_dependencies_distinguish_hosted_and_unsupported_git() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"devDependencies":{"gitpkgnotfound1":"git+https://git@github.com/testuser/test.git","gitpkgnotfound2":"git+ssh://git@some.com/testuser/test.git","gitpkgnotfound3":"git://example.com/testuser/test"}}"#.to_owned(),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert_eq!(output.suggestions[1].status, "notSupported");
    assert_eq!(output.suggestions[1].latest, None);
    assert_eq!(output.suggestions[2].status, "notSupported");
    assert_eq!(output.suggestions[2].latest, None);
    assert!(output.edits.is_empty());
}

fn local_test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "versionlens-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}
