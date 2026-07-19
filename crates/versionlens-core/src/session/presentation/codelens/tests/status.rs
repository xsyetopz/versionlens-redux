#[test]
fn code_lens_title_uses_satisfies_latest_indicator() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-left-pad-caret-1.0.0.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        }],
    );

    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[0].title, "S satisfies latest 1.1.0");
}

#[test]
fn code_lens_title_uses_latest_indicator_for_current_dependencies() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-typescript-latest.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "typescript".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"6.0.3"}}"#.to_owned(),
        }],
    );

    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[0].title, "L latest 6.0.3");
}

#[test]
fn code_lens_title_shows_fixed_git_dependencies() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("Cargo-git-dependency.toml"),
        workspace_root: None,
    };

    session.resolve_document(input.clone());
    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[0].title, "M fixed git repository");
}

#[test]
fn missing_suggestion_code_lens_is_omitted_like_upstream() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-left-pad-1.0.0.json"),
        workspace_root: None,
    };

    let output = session.analyze_document(input);

    assert!(output.code_lenses.is_empty());
}

#[test]
fn code_lens_title_preserves_configured_indicator_spacing_like_non_windows_upstream() {
    let mut indicators = test_indicators();
    indicators.updateable = "U ".to_owned();
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: indicators,
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-left-pad-1.0.0.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        }],
    );

    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[1].title, "U  latest 1.1.0");
}

pub(super) fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/presentation/codelens")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session presentation codelens fixture {}: {error}",
            path.display()
        )
    })
}
