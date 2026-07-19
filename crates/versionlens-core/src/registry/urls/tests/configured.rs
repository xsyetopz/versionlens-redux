#[test]
fn clojure_deps_edn_mvn_repos_are_used_after_builtin_repositories() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///deps.edn".to_owned(),
        language_id: "clojure".to_owned(),
        text: package_file_fixture(
            "clojure-deps-edn-mvn-repos-are-used-after-builtin-repositories.edn",
        ),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
            "https://repo.clojars.org/com/example/demo/maven-metadata.xml",
            "https://maven.example.test/releases/com/example/demo/maven-metadata.xml"
        ]
    );
}

#[test]
fn leiningen_project_clj_repositories_are_used_after_builtin_repositories() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///project.clj".to_owned(),
        language_id: "clojure".to_owned(),
        text: package_file_fixture(
            "leiningen-project-clj-repositories-are-used-after-builtin-repositories.clj",
        ),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

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
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///mix.exs".to_owned(),
        language_id: "elixir".to_owned(),
        text: package_file_fixture(
            "mix-hex-project-api-url-overrides-default-hex-registry-url.exs",
        ),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://hex.example.test/api/packages/plug"]
    );
}

#[test]
fn mix_hex_env_api_url_takes_precedence_over_project_api_url() {
    let root = temp_dir().join(format!("versionlens-hex-env-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".env"),
        "HEX_API_URL=https://hex.env.example.test/api\n",
    )
    .unwrap();

    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", root.join("mix.exs").display()),
        language_id: "elixir".to_owned(),
        text: package_file_fixture("mix-hex-env-api-url-takes-precedence-over-project-api-url.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://hex.env.example.test/api/packages/plug"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn hex_env_api_url_configures_rebar_and_gleam_registry_urls() {
    let root = temp_dir().join(format!("versionlens-hex-env-beam-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".env"),
        "HEX_API_URL=https://hex.env.example.test/api\n",
    )
    .unwrap();

    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let rebar_input = DocumentInput {
        uri: format!("file://{}", root.join("rebar.config").display()),
        language_id: "erlang".to_owned(),
        text: package_file_fixture("hex-env-api-url-configures-rebar-and-gleam-registry-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let gleam_input = DocumentInput {
        uri: format!("file://{}", root.join("gleam.toml").display()),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "hex-env-api-url-configures-rebar-and-gleam-registry-urls-2.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };

    let rebar_context = crate::registry::registry_context_from_document(&rebar_input);
    let gleam_context = crate::registry::registry_context_from_document(&gleam_input);
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

    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", root.join("rebar.config").display()),
        language_id: "erlang".to_owned(),
        text: package_file_fixture("rebar-hex-cdn-env-configures-registry-url.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://repo.example.test/api/packages/cowboy"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn rebar_packages_cdn_configures_registry_url() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///rebar.config".to_owned(),
        language_id: "erlang".to_owned(),
        text: package_file_fixture("rebar-packages-cdn-configures-registry-url.config"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://repo.project.example.test/api/packages/cowboy"]
    );
}

#[test]
fn deno_jsr_import_aliases_use_specifier_package_for_registry_urls() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: package_file_fixture(
            "deno-jsr-import-aliases-use-specifier-package-for-registry-urls.json",
        ),
        workspace_root: None,
    };
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "luca");
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("@luca/cases"));
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://jsr.io/@luca/cases/meta.json"]
    );
}
