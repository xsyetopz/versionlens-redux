#[test]
fn composer_inline_alias_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture(
                "inline-alias-dependencies-are-fixed-without-registry-updates.json",
            ), None),
        &[RegistryResponseInput::new("acme/pkg".to_owned(), Composer, r#"{"packages":{"acme/pkg":[{"version":"1.1.0"}]}}"#.to_owned())],
    );

    crate::support::tests::assert_fixed_suggestion(&output, "dev-bugfix as 1.0.x-dev");
}

#[test]
fn composer_inline_package_repository_resolves_without_registry_response() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture(
                "inline-package-repository-resolves-without-registry-response.json",
            ), None),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    crate::support::tests::assert_suggestion(&output, 0, "updateAvailable", Some("3.1.7"));
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "3.1.7");
}

#[test]
fn composer_repository_filters_route_matching_packages_only() {
    let input = DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture("repository-filters-route-matching-packages-only.json"), None);
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

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
    let input = DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture("can-disable-default-packagist-registry.json"), None);
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

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
        DocumentInput::new("file:///compose.yaml".to_owned(), "yaml".to_owned(), package_file_fixture(
                "explicit-docker-registries-return-no-match-from-mcr-shaped-responses.yaml",
            ), None),
        &[RegistryResponseInput::new("team/app".to_owned(), Docker, r#"{"results":[{"name":"2.0.0","images":[{"digest":"sha256:abc"}]}]}"#.to_owned())],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "noMatch", None);
}

#[test]
fn docker_compose_bare_build_contexts_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("docker-directory");
    let local = root.join("backend/dockerfile");
    create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(file_uri(&root.join("compose.yaml")), "yaml".to_owned(), package_file_fixture(
                "docker-compose-bare-build-contexts-resolve-as-directories.txt",
            ), None),
        &[RegistryResponseInput::new("backend/dockerfile".to_owned(), Docker, r#"{"results":[{"name":"2.0.0","images":[{"digest":"sha256:abc"}]}]}"#.to_owned())],
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
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture(
                "git-dependencies-distinguish-hosted-and-unsupported-git.json",
            ), None),
        &[],
    );

    crate::session::resolution::tests::assert_fixed_git_repository_suggestion(&output);
    assert_eq!(output.suggestions[1].status, "notSupported");
    assert_eq!(output.suggestions[1].latest, None);
    assert_eq!(output.suggestions[2].status, "notSupported");
    assert_eq!(output.suggestions[2].latest, None);
    assert!(output.edits.is_empty());
}

#[test]
fn unity_local_and_git_dependencies_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();
    let output = session.resolve_document(DocumentInput::new("file:///work/Packages/manifest.json".to_owned(), "json".to_owned(), package_file_fixture(
            "unity-local-and-git-dependencies-resolve-as-fixed-without-registry-updates.json",
        ), None));

    assert_eq!(output.suggestions.len(), 2);
    crate::support::tests::assert_all_fixed_without_edits(&output);
}
