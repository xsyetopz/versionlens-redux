use super::*;

#[test]
fn registry_context_reads_new_npm_yarn_nuget_and_gradle_overlays() {
    let workspace = TestWorkspace::new("registry-overlays");
    let npm = npm_input(&workspace, "npm/package.json");
    let yarn = npm_input(&workspace, "yarn/package.json");
    let dotnet = workspace.document("dotnet/app.csproj", "<Project />", 1);
    let gradle = workspace.document("gradle/build.gradle", "", 1);
    let configs = vec![
        workspace.document("npm/.npmrc", "registry=https://npm.overlay.test/", 2),
        workspace.document(
            "yarn/.yarnrc.yml",
            "npmRegistryServer: https://yarn.overlay.test/",
            3,
        ),
        workspace.document(
            "dotnet/NuGet.config",
            r#"<configuration><packageSources><clear /><add key="private" value="https://nuget.overlay.test/v3/index.json" /></packageSources></configuration>"#,
            4,
        ),
        workspace.document(
            "gradle/settings.gradle",
            r#"dependencyResolutionManagement {
    repositories {
        maven {
            url = uri("https://gradle.overlay.test/releases")
        }
    }
}"#,
            5,
        ),
    ];
    let session = session();
    assert!(session.set_workspace_documents(configs));

    let npm_registry = npm_context(&session, &npm);
    assert!(registry_url(&npm_registry, "demo").starts_with("https://npm.overlay.test/"));
    let yarn_context = npm_context(&session, &yarn);
    assert!(registry_url(&yarn_context, "demo").starts_with("https://yarn.overlay.test/"));

    let dotnet_context = session.registry_context(&dotnet, ManifestKind::DotnetXml);
    let mut dotnet_dependency = dependency("Demo", "1.0.0");
    dotnet_dependency.ecosystem = Ecosystem::Dotnet;
    assert_eq!(
        dotnet_context.dotnet_registry_urls(&dotnet_dependency),
        ["https://nuget.overlay.test/v3/index.json"]
    );

    let gradle_context = session.registry_context(&gradle, ManifestKind::GradleBuild);
    assert!(
        gradle_context
            .urls
            .iter()
            .any(|config| config.url == "https://gradle.overlay.test/releases")
    );
}

#[test]
fn rootless_manifests_share_unsaved_registry_overlays_without_workspace_graphs() {
    let first = TestWorkspace::new("rootless-registry-first");
    let second = TestWorkspace::new("rootless-registry-second");
    let first_input = rootless(npm_input(&first, "package.json"));
    let second_input = rootless(npm_input(&second, "package.json"));
    let documents = vec![
        rootless(first.document(".npmrc", "registry=https://first-rootless.test/", 2)),
        rootless(second.document(".npmrc", "registry=https://second-rootless.test/", 3)),
    ];
    let session = session();

    assert!(session.set_workspace_documents(documents));
    assert!(session.workspace_documents().is_empty());
    assert!(
        registry_url(&npm_context(&session, &first_input), "demo")
            .starts_with("https://first-rootless.test/")
    );
    assert!(
        registry_url(&npm_context(&session, &second_input), "demo")
            .starts_with("https://second-rootless.test/")
    );
}

#[test]
fn rootless_registry_overlay_changes_replace_context_and_auth_scope() {
    let workspace = TestWorkspace::new("rootless-registry-generation");
    let input = rootless(npm_input(&workspace, "package.json"));
    assert_registry_overlay_replacement(
        &workspace,
        &input,
        ["first-rootless.test", "second-rootless.test"],
        true,
    );
}

#[test]
fn rootless_cargo_manifest_reads_unsaved_cargo_registry_config() {
    let workspace = TestWorkspace::new("rootless-cargo-registry");
    let input = rootless(workspace.document("Cargo.toml", "[dependencies]\nserde = \"1\"", 1));
    let config = rootless(workspace.document(
        ".cargo/config.toml",
        "[source.crates-io]\nreplace-with = 'mirror'\n[source.mirror]\nregistry = 'sparse+https://cargo-rootless.test/api/'",
        2,
    ));
    let session = session();
    assert!(session.set_workspace_documents(vec![config]));
    let context = session.registry_context(&input, ManifestKind::CargoToml);
    let mut cargo_dependency = dependency("serde", "1");
    cargo_dependency.ecosystem = Ecosystem::Cargo;

    assert_eq!(
        context.registry_endpoints(&cargo_dependency)[0].url,
        "https://cargo-rootless.test/api/serde/versions"
    );
}

#[test]
fn rootless_registry_overlays_keep_the_registry_file_size_limit() {
    let workspace = TestWorkspace::new("rootless-registry-bound");
    let input = rootless(npm_input(&workspace, "package.json"));
    let oversized = rootless(workspace.document(".npmrc", &"x".repeat(1024 * 1024 + 1), 1));
    let session = session();
    assert!(session.set_workspace_documents(vec![oversized]));

    assert_registry_size_failure(&session, &input);

    let recovered =
        rootless(workspace.document(".npmrc", "registry=https://rootless-recovered.test/", 2));
    assert!(session.set_workspace_documents(vec![recovered]));
    let context = npm_context(&session, &input);
    assert!(context.failure_message().is_none());
    assert!(registry_url(&context, "demo").starts_with("https://rootless-recovered.test/"));
}

#[test]
fn escaped_document_uri_finds_an_unsaved_sibling_config() {
    let workspace = TestWorkspace::new("registry-uri");
    let directory = "project space #1";
    let config = workspace.document(
        &format!("{directory}/.npmrc"),
        "registry=https://escaped.overlay.test/",
        2,
    );
    let path = workspace.root.join(directory).join("package.json");
    let encoded_uri = format!(
        "file://{}",
        path.to_string_lossy()
            .replace(' ', "%20")
            .replace('#', "%23")
    );
    let input = npm_document(encoded_uri, workspace.root.to_string_lossy().into_owned());
    let session = session();
    session.set_workspace_documents(vec![config]);

    let context = npm_context(&session, &input);
    assert!(registry_url(&context, "demo").starts_with("https://escaped.overlay.test/"));
}

#[cfg(unix)]
#[test]
fn symlink_workspace_root_normalizes_a_new_unsaved_config() {
    use std::os::unix::fs::symlink;

    let workspace = TestWorkspace::new("registry-symlink-target");
    let alias_holder = TestWorkspace::new("registry-symlink-alias");
    let alias = alias_holder.root.join("workspace");
    symlink(&workspace.root, &alias).unwrap();
    let root = alias.to_string_lossy().into_owned();
    let config = DocumentInput::new(
        alias.join("new/.npmrc").to_string_lossy(),
        "properties",
        "registry=https://symlink.overlay.test/",
        Some(root.clone()),
    )
    .with_version(2);
    let input = npm_document(
        alias
            .join("new/package.json")
            .to_string_lossy()
            .into_owned(),
        root,
    );
    let context = npm_context_with_documents(&input, vec![config]);
    assert!(registry_url(&context, "demo").starts_with("https://symlink.overlay.test/"));
}

#[test]
fn registry_overlay_changes_replace_context_and_cache_scope() {
    let workspace = TestWorkspace::new("registry-generation");
    let input = npm_input(&workspace, "package.json");
    assert_registry_overlay_replacement(&workspace, &input, ["first.test", "second.test"], false);
}

#[test]
fn disk_registry_context_is_stable_until_workspace_invalidation() {
    let workspace = TestWorkspace::new("registry-disk-cache");
    let input = npm_input(&workspace, "package.json");
    let session = session();

    let missing = npm_context(&session, &input);
    assert!(
        missing
            .registry_endpoints(&dependency("demo", "1.0.0"))
            .is_empty()
    );
    workspace.write(".npmrc", "registry=https://before.test/");
    let cached_miss = npm_context(&session, &input);
    assert!(
        cached_miss
            .registry_endpoints(&dependency("demo", "1.0.0"))
            .is_empty()
    );

    session.invalidate_workspace();
    let before = npm_context(&session, &input);
    workspace.write(".npmrc", "registry=https://after.test/");
    let cached_content = npm_context(&session, &input);
    assert_eq!(
        registry_url(&before, "demo"),
        registry_url(&cached_content, "demo")
    );

    session.invalidate_workspace();
    let after = npm_context(&session, &input);
    assert!(registry_url(&after, "demo").starts_with("https://after.test/"));
}

#[test]
fn parsed_registry_context_cache_evicts_old_input_snapshots() {
    let workspace = TestWorkspace::new("registry-context-bound");
    let session = session();
    for version in 0..=MAX_REGISTRY_CONTEXTS {
        let input = workspace.document(
            "package.json",
            &format!(r#"{{"name":"app","version":"{version}"}}"#),
            version as u64,
        );
        npm_context(&session, &input);
    }

    let state = session
        .storage_state
        .workspace
        .lock()
        .unwrap_or_else(crate::recover_poison);
    assert_eq!(state.registry_contexts.len(), MAX_REGISTRY_CONTEXTS);
}

#[test]
fn sparse_oversized_registry_file_fails_until_workspace_invalidation() {
    let workspace = TestWorkspace::new("registry-sparse-bound");
    let input = npm_input(&workspace, "package.json");
    let config_path = workspace.root.join(".npmrc");
    let file = std::fs::File::create(&config_path).unwrap();
    file.set_len(1024 * 1024 + 1).unwrap();
    let session = session();

    assert_registry_size_failure(&session, &input);
    std::fs::write(&config_path, "registry=https://recovered.test/").unwrap();
    assert!(npm_context(&session, &input).failure_message().is_some());

    session.invalidate_workspace();
    let recovered = npm_context(&session, &input);
    assert!(recovered.failure_message().is_none());
    assert!(registry_url(&recovered, "demo").starts_with("https://recovered.test/"));
}

#[test]
fn registry_failure_does_not_replace_an_unrelated_cached_context() {
    let workspace = TestWorkspace::new("registry-failure-cache");
    workspace.write("healthy/.npmrc", "registry=https://cached.test/");
    let mut input = npm_input(&workspace, "healthy/package.json");
    input.uri = crate::workspace_file_uri(&workspace.root.join("healthy/package.json")).unwrap();
    let session = crate::support::tests::test_session(false);
    let successful = session.resolve_document_with_responses(
        input.clone(),
        &[crate::RegistryResponseInput::new(
            "demo".to_owned(),
            Ecosystem::Npm,
            r#"{"versions":{"1.0.0":{},"2.0.0":{}}}"#.to_owned(),
        )],
    );
    assert_ne!(successful.suggestions[0].status, "error");

    let oversized = workspace.document("broken/.npmrc", &"x".repeat(1024 * 1024 + 1), 2);
    assert!(
        session
            .registry_context(&oversized, ManifestKind::NpmPackageJson)
            .failure_message()
            .is_some()
    );
    let healthy = npm_context(&session, &input);
    assert!(healthy.failure_message().is_none());
    assert!(registry_url(&healthy, "demo").starts_with("https://cached.test/"));
    let resolved = session.resolve_document(input);
    assert_ne!(resolved.suggestions[0].status, "error");
}

#[test]
fn three_hundred_member_registry_contexts_fit_in_one_snapshot() {
    let workspace = TestWorkspace::new("registry-many-members");
    let session = session();

    for index in 0..300 {
        let input = npm_input(&workspace, &format!("packages/member-{index}/package.json"));
        assert!(
            npm_context(&session, &input).failure_message().is_none(),
            "registry context {index} exceeded the shared snapshot budget"
        );
    }
}

#[test]
fn local_dependency_stays_fresh_when_registry_configuration_fails() {
    let (workspace, consumer) = local_member_workspace("local-registry-failure");
    let config = std::fs::File::create(workspace.root.join(".npmrc")).unwrap();
    config.set_len(1024 * 1024 + 1).unwrap();
    let session = crate::support::tests::test_session(false);
    let local_dependency = local_dependency(&session, &consumer);
    assert_eq!(local_dependency.name, "local-package");
    assert_eq!(local_dependency.requirement, "^1.0.0");
    assert_eq!(local_dependency.ecosystem, Ecosystem::Npm);
    let graph = session.workspace_graph(&consumer);
    assert!(
        graph
            .resolve(&dependency("local-package", "^1.0.0"))
            .is_some()
    );
    assert!(graph.resolve(&local_dependency).is_some());

    let output = session.resolve_document(consumer.clone());

    let suggestion = output
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == "local-package")
        .unwrap();
    assert_ne!(suggestion.status, "error");
    assert_eq!(suggestion.latest.as_deref(), Some("1.0.0"));
    assert!(npm_context(&session, &consumer).failure_message().is_some());
    assert!(session.document_is_fresh(&consumer));
}
