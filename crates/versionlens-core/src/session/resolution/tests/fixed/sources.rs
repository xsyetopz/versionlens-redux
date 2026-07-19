use versionlens_model::Ecosystem::{Cargo, Docker};
#[test]
fn ruby_path_block_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("ruby-path-block-directory");
    let app = root.join("app");
    let local = app.join("vendor/local");
    create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&app.join("Gemfile")),
            language_id: "ruby".to_owned(),
            text: package_file_fixture("ruby-path-block-dependencies-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local_one".to_owned(),
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
fn git_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("git-dependencies-are-fixed.toml"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "remote".to_owned(),
            ecosystem: Cargo,
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
    let root = temp_dir().join(format!("versionlens-cargo-registry-{}", id()));
    create_dir_all(root.join(".cargo")).unwrap();
    write(
        root.join(".cargo/config.toml"),
        "[registries.private]\nindex = 'https://cargo.example.test/api/'\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("Cargo.toml").display()),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "cargo-registry-dependencies-use-workspace-cargo-config-urls.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "private");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://cargo.example.test/api/private/versions"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn cargo_renamed_registry_dependencies_use_package_identity_for_lookup() {
    let root = temp_dir().join(format!("versionlens-cargo-renamed-registry-{}", id()));
    create_dir_all(root.join(".cargo")).unwrap();
    write(
        root.join(".cargo/config.toml"),
        "[registries.private]\nindex = 'https://cargo.example.test/api/'\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("Cargo.toml").display()),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "cargo-renamed-registry-dependencies-use-package-identity-for-lookup.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

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

#[test]
fn cargo_crates_io_source_replacement_uses_workspace_cargo_config_url() {
    let root = temp_dir().join(format!("versionlens-cargo-replace-{}", id()));
    create_dir_all(root.join(".cargo")).unwrap();
    write(
        root.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = 'mirror'\n[source.mirror]\nregistry = 'sparse+https://mirror.example.test/api/'\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("Cargo.toml").display()),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "cargo-crates-io-source-replacement-uses-workspace-cargo-config-url.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

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
        DocumentInput {
            uri: "file:///repo/member/Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture(
                "cargo-workspace-inherited-dependencies-do-not-create-registry-updates.toml",
            ),
            workspace_root: Some("/repo".to_owned()),
        },
        &[
            RegistryResponseInput {
                package: "regex".to_owned(),
                ecosystem: Cargo,
                body: r#"{"versions":[{"num":"9.9.9"}]}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "cc".to_owned(),
                ecosystem: Cargo,
                body: r#"{"versions":[{"num":"9.9.9"}]}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "rand".to_owned(),
                ecosystem: Cargo,
                body: r#"{"versions":[{"num":"9.9.9"}]}"#.to_owned(),
            },
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

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-dependencies-use-workspace-bunfig-registry-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
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
        context.auth_headers_for_url(Npm, "https://scope.example.test/npm/@scope%2fpkg")[0].value,
        "Bearer secret"
    );

    remove_dir_all(root).unwrap();
}
