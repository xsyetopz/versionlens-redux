use versionlens_parsers::Ecosystem::Dotnet;
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
        text: package_file_fixture("clojure-deps-edn-mvn-repos-are-used-after-builtin-repositories.edn"),
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
        text: package_file_fixture("leiningen-project-clj-repositories-are-used-after-builtin-repositories.clj"),
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
        text: package_file_fixture("mix-hex-project-api-url-overrides-default-hex-registry-url.exs"),
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
    let root =
        temp_dir().join(format!("versionlens-hex-env-beam-{}", id()));
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
        text: package_file_fixture("hex-env-api-url-configures-rebar-and-gleam-registry-urls-2.txt"),
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
        text: package_file_fixture("deno-jsr-import-aliases-use-specifier-package-for-registry-urls.json"),
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

#[test]
fn dotnet_sources_are_service_indexes_not_package_urls() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Dotnet,
                url: "https://nuget.example.test/v3/index.json".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: package_file_fixture("dotnet-sources-are-service-indexes-not-package-urls.csproj"),
        workspace_root: None,
    };
    let dependencies = session.dependencies(&input);

    let urls = session.registry_urls(&dependencies[0]);
    assert_eq!(urls[0], "https://nuget.example.test/v3/index.json");
    assert!(
        urls.iter()
            .all(|url| !url.contains("newtonsoft.json/index.json"))
    );
}

#[test]
fn dotnet_documents_use_workspace_nuget_config_sources() {
    let root =
        temp_dir().join(format!("versionlens-nuget-config-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("NuGet.config"),
        r#"
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
    <add key="private" value="https://nuget.example.test/v3/index.json" />
  </packageSources>
  <disabledPackageSources>
    <add key="nuget.org" value="true" />
  </disabledPackageSources>
</configuration>
"#,
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
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture("dotnet-documents-use-workspace-nuget-config-sources.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://nuget.example.test/v3/index.json"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn paket_dependencies_use_declared_source_urls() {
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
        uri: "file:///paket.dependencies".to_owned(),
        language_id: "plaintext".to_owned(),
        text: package_file_fixture("paket-dependencies-use-declared-source-urls.dependencies"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://nuget.example.test/v3/index.json"]
    );
}

#[test]
fn dotnet_child_nuget_config_clear_removes_workspace_sources() {
    let root = temp_dir().join(format!(
        "versionlens-nuget-config-clear-{}",
        id()
    ));
    let app = root.join("src");
    create_dir_all(&app).unwrap();
    write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><clear /><add key="child" value="https://child.example.test/v3/index.json" /></packageSources></configuration>"#,
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
        uri: format!("file://{}", app.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture("dotnet-child-nuget-config-clear-removes-workspace-sources.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://child.example.test/v3/index.json"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_child_nuget_config_remove_does_not_delete_inherited_cli_sources() {
    let root = temp_dir().join(format!(
        "versionlens-nuget-config-remove-{}",
        id()
    ));
    let app = root.join("src");
    create_dir_all(&app).unwrap();
    write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /><add key="keep" value="https://keep.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><remove key="root" /></packageSources></configuration>"#,
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
        uri: format!("file://{}", app.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture("dotnet-child-nuget-config-remove-does-not-delete-inherited-cli-sources.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://root.example.test/v3/index.json",
            "https://keep.example.test/v3/index.json"
        ]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_reads_intermediate_ancestors_nearest_first() {
    let root = temp_dir().join(format!(
        "versionlens-nuget-config-ancestors-{}",
        id()
    ));
    let src = root.join("src");
    let app = src.join("app");
    create_dir_all(&app).unwrap();
    write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    write(
        src.join("NuGet.config"),
        r#"<configuration><packageSources><add key="src" value="https://src.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><add key="app" value="https://app.example.test/v3/index.json" /></packageSources></configuration>"#,
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
        uri: format!("file://{}", app.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture("dotnet-nuget-config-reads-intermediate-ancestors-nearest-first.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://app.example.test/v3/index.json",
            "https://src.example.test/v3/index.json",
            "https://root.example.test/v3/index.json"
        ]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_ignores_local_file_sources_for_suggestions() {
    let root = temp_dir().join(format!(
        "versionlens-nuget-config-local-source-{}",
        id()
    ));
    create_dir_all(root.join("packages")).unwrap();
    write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="local" value="./packages" /></packageSources></configuration>"#,
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
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture("dotnet-nuget-config-ignores-local-file-sources-for-suggestions.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert!(
        session
            .registry_urls_with_context(&dependencies[0], &context)
            .is_empty()
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_supplies_request_scoped_auth_headers() {
    let root = temp_dir().join(format!(
        "versionlens-nuget-config-auth-{}",
        id()
    ));
    create_dir_all(&root).unwrap();
    write(
        root.join("NuGet.config"),
        r#"
<configuration>
  <packageSources>
    <add key="private" value="https://nuget.example.test/v3/index.json" />
  </packageSources>
  <packageSourceCredentials>
    <private>
      <add key="Username" value="user" />
      <add key="ClearTextPassword" value="pass" />
    </private>
  </packageSourceCredentials>
</configuration>
"#,
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture("dotnet-nuget-config-supplies-request-scoped-auth-headers.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let service_index_headers = context.auth_headers_for_url(
        Dotnet,
        "https://nuget.example.test/v3/index.json",
    );
    let package_headers = context.auth_headers_for_url(
        Dotnet,
        "https://nuget.example.test/v3-flatcontainer/newtonsoft.json/index.json",
    );
    let other_headers = context.auth_headers_for_url(
        Dotnet,
        "https://other.example.test/v3/index.json",
    );

    assert_eq!(service_index_headers.len(), 1);
    assert_eq!(service_index_headers[0].value, "Basic dXNlcjpwYXNz");
    assert_eq!(package_headers.len(), 1);
    assert_eq!(package_headers[0].value, "Basic dXNlcjpwYXNz");
    assert!(other_headers.is_empty());

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_package_source_mapping_filters_sources() {
    let root = temp_dir().join(format!(
        "versionlens-nuget-config-mapping-{}",
        id()
    ));
    create_dir_all(&root).unwrap();
    write(
        root.join("NuGet.config"),
        r#"
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
    <add key="private" value="https://nuget.example.test/v3/index.json" />
  </packageSources>
  <packageSourceMapping>
    <packageSource key="nuget.org">
      <package pattern="Newtonsoft.*" />
    </packageSource>
    <packageSource key="private">
      <package pattern="Contoso.*" />
    </packageSource>
  </packageSourceMapping>
</configuration>
"#,
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
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture("dotnet-nuget-config-package-source-mapping-filters-sources.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://api.nuget.org/v3/index.json"]
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec!["https://nuget.example.test/v3/index.json"]
    );

    remove_dir_all(root).unwrap();
}
