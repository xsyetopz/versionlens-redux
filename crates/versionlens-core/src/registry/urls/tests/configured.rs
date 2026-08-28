#[test]
fn clojure_deps_edn_mvn_repos_are_used_after_builtin_repositories() {
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new("file:///deps.edn".to_owned(), "clojure".to_owned(), package_file_fixture(
            "clojure-deps-edn-mvn-repos-are-used-after-builtin-repositories.edn",
        ), None);
    assert_first_registry_urls(&session, &input, &["https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
            "https://repo.clojars.org/com/example/demo/maven-metadata.xml",
            "https://maven.example.test/releases/com/example/demo/maven-metadata.xml"]);
}

#[test]
fn leiningen_project_clj_repositories_are_used_after_builtin_repositories() {
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new("file:///project.clj".to_owned(), "clojure".to_owned(), package_file_fixture(
            "leiningen-project-clj-repositories-are-used-after-builtin-repositories.clj",
        ), None);
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec![
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
            "https://repo.clojars.org/com/example/demo/maven-metadata.xml",
            "https://maven.example.test/releases/com/example/demo/maven-metadata.xml"
        ]
    );
}

#[test]
fn mix_hex_project_api_url_overrides_default_hex_registry_url() {
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new("file:///mix.exs".to_owned(), "elixir".to_owned(), package_file_fixture(
            "mix-hex-project-api-url-overrides-default-hex-registry-url.exs",
        ), None);
    assert_first_registry_urls(&session, &input, &["https://hex.example.test/api/packages/plug"]);
}

#[test]
fn mix_hex_env_api_url_takes_precedence_over_project_api_url() {
    let root = workspace_with_env(
        "hex-env",
        "HEX_API_URL=https://hex.env.example.test/api\n",
    );

    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(format!("file://{}", root.join("mix.exs").display()), "elixir".to_owned(), package_file_fixture("mix-hex-env-api-url-takes-precedence-over-project-api-url.txt"), Some(root.to_string_lossy().into_owned()));
    assert_first_registry_urls(&session, &input, &["https://hex.env.example.test/api/packages/plug"]);

    remove_dir_all(root).unwrap();
}

#[test]
fn hex_env_api_url_configures_rebar_and_gleam_registry_urls() {
    let root = workspace_with_env(
        "hex-env-beam",
        "HEX_API_URL=https://hex.env.example.test/api\n",
    );

    let session = crate::support::tests::test_session(false);
    let rebar_input = DocumentInput::new(format!("file://{}", root.join("rebar.config").display()), "erlang".to_owned(), package_file_fixture("hex-env-api-url-configures-rebar-and-gleam-registry-urls.txt"), Some(root.to_string_lossy().into_owned()));
    let gleam_input = DocumentInput::new(format!("file://{}", root.join("gleam.toml").display()), "toml".to_owned(), package_file_fixture(
            "hex-env-api-url-configures-rebar-and-gleam-registry-urls-2.txt",
        ), Some(root.to_string_lossy().into_owned()));

    let rebar_context = crate::registry::RegistryContext::from_document(&rebar_input);
    let gleam_context = crate::registry::RegistryContext::from_document(&gleam_input);
    let rebar_dependencies = session.dependencies(&rebar_input);
    let gleam_dependencies = session.dependencies(&gleam_input);

    assert_eq!(
        session.registry_urls_with_context(&rebar_dependencies[0], &rebar_context),
        vec!["https://hex.env.example.test/api/packages/cowboy"]
    );
    assert_eq!(
        session.registry_urls_with_context(&gleam_dependencies[0], &gleam_context),
        vec!["https://hex.env.example.test/api/packages/gleam_stdlib"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn rebar_hex_cdn_env_configures_registry_url() {
    let root = temp_dir().join(format!("versionlens-hex-cdn-env-{}", id()));
    create_dir_all(&root).unwrap();
    write(root.join(".env"), "HEX_CDN=https://repo.example.test\n").unwrap();

    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(format!("file://{}", root.join("rebar.config").display()), "erlang".to_owned(), package_file_fixture("rebar-hex-cdn-env-configures-registry-url.txt"), Some(root.to_string_lossy().into_owned()));
    assert_first_registry_urls(&session, &input, &["https://repo.example.test/api/packages/cowboy"]);

    remove_dir_all(root).unwrap();
}

#[test]
fn rebar_packages_cdn_configures_registry_url() {
    let session = crate::support::tests::test_session(false);
    let input = registry_input("file:///rebar.config", "erlang", "rebar-packages-cdn-configures-registry-url.config");
    assert_first_registry_urls(&session, &input, &["https://repo.project.example.test/api/packages/cowboy"]);
}

#[test]
fn deno_jsr_import_aliases_use_specifier_package_for_registry_urls() {
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new("file:///deno.json".to_owned(), "jsonc".to_owned(), package_file_fixture(
            "deno-jsr-import-aliases-use-specifier-package-for-registry-urls.json",
        ), None);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "luca");
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("@luca/cases"));
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://jsr.io/@luca/cases/meta.json"]
    );
}
fn workspace_with_env(name: &str, contents: &str) -> std::path::PathBuf {
    let root = temp_dir().join(format!("versionlens-{name}-{}", id()));
    create_dir_all(&root).unwrap();
    write(root.join(".env"), contents).unwrap();
    root
}
