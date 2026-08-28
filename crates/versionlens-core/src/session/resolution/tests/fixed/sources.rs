#[test]
fn ruby_path_block_dependencies_resolve_as_directories() {
    assert_ruby_path_dependency_fixture(
        "ruby-path-block-dependencies-resolve-as-directories.txt",
        "local_one",
        "ruby-path-block-directory",
    );
}

#[test]
fn git_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///Cargo.toml".to_owned(), "toml".to_owned(), package_file_fixture("git-dependencies-are-fixed.toml"), None),
        &[RegistryResponseInput::new("remote".to_owned(), Cargo, r#"{"crate":{"max_version":"9.9.9"}}"#.to_owned())],
    );

    crate::support::tests::assert_fixed_git_repository(&output);
}

#[test]
fn cargo_registry_dependencies_use_workspace_cargo_config_urls() {
    let (root, input) = cargo_registry_input(
        "versionlens-cargo-registry",
        "cargo-registry-dependencies-use-workspace-cargo-config-urls.txt",
    );
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

    assert_eq!(dependencies[0].name, "private");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://cargo.example.test/api/private/versions"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn cargo_renamed_registry_dependencies_use_package_identity_for_lookup() {
    let (root, input) = cargo_registry_input(
        "versionlens-cargo-renamed-registry",
        "cargo-renamed-registry-dependencies-use-package-identity-for-lookup.txt",
    );
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

    assert_eq!(dependencies[0].name, "local_name");
    assert_eq!(
        dependencies[0].hosted_name.as_deref(),
        Some("registry-name")
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://cargo.example.test/api/registry-name/versions"]
    );

    remove_dir_all(root).unwrap();
}

fn cargo_registry_input(
    root_name: &str,
    fixture: &str,
) -> (std::path::PathBuf, DocumentInput) {
    let root = temp_dir().join(format!("{root_name}-{}", id()));
    create_dir_all(root.join(".cargo")).unwrap();
    write(
        root.join(".cargo/config.toml"),
        "[registries.private]\nindex = 'https://cargo.example.test/api/'\n",
    )
    .unwrap();
    let input = DocumentInput::new(
        format!("file://{}", root.join("Cargo.toml").display()),
        "toml".to_owned(),
        package_file_fixture(fixture),
        Some(root.to_string_lossy().into_owned()),
    );
    (root, input)
}

#[test]
fn cargo_crates_io_source_replacement_uses_workspace_cargo_config_url() {
    let root = temp_dir().join(format!("versionlens-cargo-replace-{}", id()));
    create_dir_all(root.join(".cargo")).unwrap();
    write(
        root.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = 'mirror'\n[source.mirror]\nregistry = 'sparse+https://mirror.example.test/api/'\n",
    )
    .unwrap();

    let input = DocumentInput::new(format!("file://{}", root.join("Cargo.toml").display()), "toml".to_owned(), package_file_fixture(
            "cargo-crates-io-source-replacement-uses-workspace-cargo-config-url.txt",
        ), Some(root.to_string_lossy().into_owned()));
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

    assert_eq!(dependencies[0].name, "serde");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://mirror.example.test/api/serde/versions"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn cargo_workspace_inherited_dependencies_do_not_create_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/member/Cargo.toml".to_owned(), "toml".to_owned(), package_file_fixture(
                "cargo-workspace-inherited-dependencies-do-not-create-registry-updates.toml",
            ), Some("/repo".to_owned())),
        &[
            RegistryResponseInput::new("regex".to_owned(), Cargo, r#"{"versions":[{"num":"9.9.9"}]}"#.to_owned()),
            RegistryResponseInput::new("cc".to_owned(), Cargo, r#"{"versions":[{"num":"9.9.9"}]}"#.to_owned()),
            RegistryResponseInput::new("rand".to_owned(), Cargo, r#"{"versions":[{"num":"9.9.9"}]}"#.to_owned()),
        ],
    );

    assert_eq!(output.suggestions.len(), 3);
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status == "fixed")
    );
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.latest.as_deref() == Some("workspace:true"))
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_dependencies_use_workspace_bunfig_registry_urls() {
    let root = temp_dir().join(format!("versionlens-bunfig-registry-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("bunfig.toml"),
        "[install]\nregistry = 'https://${REGISTRY_HOST}/npm'\n[install.scopes]\n'@scope' = { url = 'https://${SCOPE_HOST}/npm', token = '${BUN_SCOPE_TOKEN}' }\n",
    )
    .unwrap();
    write(
        root.join(".env"),
        "REGISTRY_HOST=registry.example.test\nSCOPE_HOST=scope.example.test\nBUN_SCOPE_TOKEN=secret\n",
    )
    .unwrap();

    let input = DocumentInput::new(format!("file://{}", root.join("package.json").display()), "json".to_owned(), package_file_fixture("dependencies-use-workspace-bunfig-registry-urls.txt"), Some(root.to_string_lossy().into_owned()));
    let (_session, context, _dependencies) =
        crate::session::resolution::tests::registry_case_with_expected_urls(
            &input,
            &[
                &["https://registry.example.test/npm/left-pad"],
                &["https://scope.example.test/npm/@scope%2fpkg"],
            ],
        );
    assert_eq!(
        context.auth_headers_for_url(Npm, "https://scope.example.test/npm/@scope%2fpkg")[0].value,
        "Bearer secret"
    );

    remove_dir_all(root).unwrap();
}
