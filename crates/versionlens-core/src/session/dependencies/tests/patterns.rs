#[test]
fn configured_file_pattern_classifies_custom_composer_manifest() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ComposerJson,
                pattern: "**/acme.composer.json".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("acme.composer.json"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].ecosystem, "composer");
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_brace_alternatives() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ComposerJson,
                pattern: "**/{composer.json,acme.composer.json}".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("acme.composer.json"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_workspace_relative_recursive_segments() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ComposerJson,
                pattern: "packages/**/acme.composer.json".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/packages/backend/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("acme.composer.json"),
        workspace_root: Some("/workspace".to_owned()),
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_character_classes() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ComposerJson,
                pattern: "**/acme.composer.jso[n]".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("acme.composer.json"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_character_class_ranges() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ComposerJson,
                pattern: "**/acme.composer.jso[m-o]".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("acme.composer.json"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_negated_character_classes() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ComposerJson,
                pattern: "**/acme.composer.jso[!x]".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("acme.composer.json"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_micromatch_extglob_alternatives() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ComposerJson,
                pattern: "**/@(composer|acme.composer).json".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("acme.composer.json"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_docker_file_pattern_routes_non_yaml_matches_to_dockerfile_parser() {
    let session = session_with_file_pattern(FilePatternConfig {
        manifest_kind: DockerComposeYaml,
        pattern: "**/Containerfile".to_owned(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/Containerfile".to_owned(),
        language_id: "plaintext".to_owned(),
        text: package_file_fixture("Containerfile"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].ecosystem, "docker");
    assert_eq!(output.dependencies[0].group, "FROM");
    assert_eq!(output.dependencies[0].name, "node");
    assert_eq!(output.dependencies[0].requirement, "20");
}

#[test]
fn configured_pypi_file_pattern_routes_non_txt_matches_to_toml_parser() {
    let session = session_with_file_pattern(FilePatternConfig {
        manifest_kind: PythonRequirementsTxt,
        pattern: "**/pyproject-prod.toml".to_owned(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/pyproject-prod.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("pyproject-prod.toml"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].ecosystem, "pypi");
    assert_eq!(output.dependencies[0].group, "project.dependencies");
    assert_eq!(output.dependencies[0].name, "requests");
    assert_eq!(output.dependencies[0].requirement, "==2.32.0");
}

#[test]
fn configured_dub_file_pattern_routes_sdl_matches_to_sdl_parser() {
    let session = session_with_file_pattern(FilePatternConfig {
        manifest_kind: DubJson,
        pattern: "**/*.sdl".to_owned(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/dub.sdl".to_owned(),
        language_id: "plaintext".to_owned(),
        text: package_file_fixture("dub.sdl"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].ecosystem, "dub");
    assert_eq!(output.dependencies[0].group, "dependencies");
    assert_eq!(output.dependencies[0].name, "vibe-d");
    assert_eq!(output.dependencies[0].requirement, "~>0.9.7");
}
