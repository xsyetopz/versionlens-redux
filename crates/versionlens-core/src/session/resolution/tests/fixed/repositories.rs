#[test]
fn composer_inline_alias_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "composer-inline-alias-dependencies-are-fixed-without-registry-updates.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "acme/pkg".to_owned(),
            ecosystem: Composer,
            body: r#"{"packages":{"acme/pkg":[{"version":"1.1.0"}]}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("dev-bugfix as 1.0.x-dev")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn composer_inline_package_repository_resolves_without_registry_response() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "composer-inline-package-repository-resolves-without-registry-response.json",
            ),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("3.1.7"));
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "3.1.7");
}

#[test]
fn composer_repository_filters_route_matching_packages_only() {
    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("composer-repository-filters-route-matching-packages-only.json"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
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
        text: package_file_fixture("composer-can-disable-default-packagist-registry.json"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
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
            text: package_file_fixture(
                "explicit-docker-registries-return-no-match-from-mcr-shaped-responses.yaml",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "team/app".to_owned(),
            ecosystem: Docker,
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
    create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&root.join("compose.yaml")),
            language_id: "yaml".to_owned(),
            text: package_file_fixture(
                "docker-compose-bare-build-contexts-resolve-as-directories.txt",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "backend/dockerfile".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"2.0.0","images":[{"digest":"sha256:abc"}]}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("backend/dockerfile")
    );
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn npm_git_dependencies_distinguish_hosted_and_unsupported_git() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "npm-git-dependencies-distinguish-hosted-and-unsupported-git.json",
            ),
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
    let root = temp_dir().join(format!(
        "versionlens-{name}-{}-{}",
        id(),
        crate::system_time_now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    create_dir_all(&root).unwrap();
    root
}

fn file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

#[test]
fn unity_local_and_git_dependencies_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();
    let output = session.resolve_document(DocumentInput {
        uri: "file:///work/Packages/manifest.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "unity-local-and-git-dependencies-resolve-as-fixed-without-registry-updates.json",
        ),
        workspace_root: None,
    });

    assert_eq!(output.suggestions.len(), 2);
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status == "fixed")
    );
    assert!(output.edits.is_empty());
}
