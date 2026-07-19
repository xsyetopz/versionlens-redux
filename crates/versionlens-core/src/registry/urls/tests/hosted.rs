#[test]
fn hosted_dependencies_use_hosted_registry_url() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Pub,
                url: "https://pub.dev/api/packages".to_owned(),
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
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("hosted-dependencies-use-hosted-registry-url.yaml"),
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
        vec!["https://pub.example.test/api/packages/hosted_alias"]
    );
}

#[test]
fn docker_compose_explicit_registry_uses_oci_registry_url() {
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
        uri: "file:///compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("docker-compose-explicit-registry-uses-oci-registry-url.yaml"),
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
        vec!["https://registry.example.test/v2/team/app/tags/list"]
    );
}
