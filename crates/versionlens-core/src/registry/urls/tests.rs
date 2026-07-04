use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem};

use crate::{
    ProviderSettings, RegistryUrlConfig, SessionConfig, SuggestionIndicators, VersionLensSession,
};

#[test]
fn hosted_dependencies_use_hosted_registry_url() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Ecosystem::Pub,
                url: "https://pub.dev/api/packages".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "dependencies:\n  hosted_dep:\n    version: 1.0.0\n    hosted:\n      name: hosted_alias\n      url: https://pub.example.test/\n"
            .to_owned(),
        workspace_root: None,
    };
    let dependencies = session.dependencies(&input);
    let output = session.analyze_document(input);

    assert_eq!(
        output.dependencies[0].hosted_url.as_deref(),
        Some("https://pub.example.test/")
    );
    assert_eq!(
        output.dependencies[0].hosted_name.as_deref(),
        Some("hosted_alias")
    );
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://pub.example.test//hosted_dep"]
    );
}

#[test]
fn docker_compose_explicit_registry_uses_mcr_registry_url() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: "file:///compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "services:\n  app:\n    image: registry.example.test/team/app:1.0.0\n".to_owned(),
        workspace_root: None,
    };
    let dependencies = session.dependencies(&input);
    let output = session.analyze_document(input);

    assert_eq!(output.dependencies[0].name, "team/app");
    assert_eq!(
        output.dependencies[0].hosted_url.as_deref(),
        Some("registry.example.test")
    );
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://mcr.microsoft.com/api/v1/catalog/team/app/tags?reg=mar"]
    );
}

#[test]
fn deno_jsr_import_aliases_use_specifier_package_for_registry_urls() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: r#"{"imports":{"luca":"jsr:@luca/cases@1.0.0"}}"#.to_owned(),
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
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Ecosystem::Dotnet,
                url: "https://nuget.example.test/v3/index.json".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: "file:///app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3" /></ItemGroup></Project>"#
            .to_owned(),
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
        std::env::temp_dir().join(format!("versionlens-nuget-config-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
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

    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://nuget.example.test/v3/index.json"]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_child_nuget_config_clear_removes_workspace_sources() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-nuget-config-clear-{}",
        std::process::id()
    ));
    let app = root.join("src");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    std::fs::write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><clear /><add key="child" value="https://child.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();

    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", app.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://child.example.test/v3/index.json"]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_child_nuget_config_remove_does_not_delete_inherited_cli_sources() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-nuget-config-remove-{}",
        std::process::id()
    ));
    let app = root.join("src");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /><add key="keep" value="https://keep.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    std::fs::write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><remove key="root" /></packageSources></configuration>"#,
    )
    .unwrap();

    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", app.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://root.example.test/v3/index.json",
            "https://keep.example.test/v3/index.json"
        ]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_reads_intermediate_ancestors_nearest_first() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-nuget-config-ancestors-{}",
        std::process::id()
    ));
    let src = root.join("src");
    let app = src.join("app");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    std::fs::write(
        src.join("NuGet.config"),
        r#"<configuration><packageSources><add key="src" value="https://src.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    std::fs::write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><add key="app" value="https://app.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();

    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", app.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://app.example.test/v3/index.json",
            "https://src.example.test/v3/index.json",
            "https://root.example.test/v3/index.json"
        ]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_ignores_local_file_sources_for_suggestions() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-nuget-config-local-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(root.join("packages")).unwrap();
    std::fs::write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="local" value="./packages" /></packageSources></configuration>"#,
    )
    .unwrap();

    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = session.dependencies(&input);

    assert!(
        session
            .registry_urls_with_context(&dependencies[0], &context)
            .is_empty()
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_supplies_request_scoped_auth_headers() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-nuget-config-auth-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
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
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let service_index_headers = context.auth_headers_for_url(
        Ecosystem::Dotnet,
        "https://nuget.example.test/v3/index.json",
    );
    let package_headers = context.auth_headers_for_url(
        Ecosystem::Dotnet,
        "https://nuget.example.test/v3-flatcontainer/newtonsoft.json/index.json",
    );
    let other_headers = context.auth_headers_for_url(
        Ecosystem::Dotnet,
        "https://other.example.test/v3/index.json",
    );

    assert_eq!(service_index_headers.len(), 1);
    assert_eq!(service_index_headers[0].value, "Basic dXNlcjpwYXNz");
    assert_eq!(package_headers.len(), 1);
    assert_eq!(package_headers[0].value, "Basic dXNlcjpwYXNz");
    assert!(other_headers.is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_package_source_mapping_filters_sources() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-nuget-config-mapping-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
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

    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", root.join("app.csproj").display()),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.3" /><PackageReference Include="Contoso.Core" Version="1.0.0" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://api.nuget.org/v3/index.json"]
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec!["https://nuget.example.test/v3/index.json"]
    );

    std::fs::remove_dir_all(root).unwrap();
}
