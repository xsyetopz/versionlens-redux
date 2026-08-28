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
    let input = registry_input("file:///pubspec.yaml", "yaml", "hosted-dependencies-use-hosted-registry-url.yaml");
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
    let session = crate::support::tests::test_session(false);
    let input = registry_input("file:///compose.yaml", "yaml", "docker-compose-explicit-registry-uses-oci-registry-url.yaml");
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

#[test]
fn github_actions_use_repository_identity_with_default_github_api_base() {
    let session = github_actions_session(crate::default());
    let input = github_actions_input();
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, GitHub);
    assert_eq!(dependencies[0].name, "actions/checkout");
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("actions/checkout"));
    assert_eq!(dependencies[0].hosted_url, None);
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/actions/checkout/tags"]
    );
}

#[test]
fn github_actions_use_repository_identity_with_custom_github_api_base() {
    let session = github_actions_session(ProviderSettings {
        registry_urls: vec![RegistryUrlConfig {
            ecosystem: GitHub,
            url: "https://github.example.test/api/repos/".to_owned(),
        }],
        ..crate::default()
    });
    let input = github_actions_input();
    let dependencies = session.dependencies(&input);

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://github.example.test/api/repos/actions/checkout/tags"]
    );
}

fn github_actions_input() -> DocumentInput {
    registry_input("file:///work/.github/workflows/ci.yml", "yaml", "github-actions-use-configured-registry-url.yaml")
}

fn github_actions_session(providers: ProviderSettings) -> crate::VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers,
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}
