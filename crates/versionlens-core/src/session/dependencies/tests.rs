use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem, ManifestKind};

use crate::{
    EnabledProviderConfig, FilePatternConfig, ProviderSettings, SessionConfig,
    SuggestionIndicators, VersionLensSession,
};

#[test]
fn disabled_providers_are_filtered_in_rust() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![EnabledProviderConfig {
            ecosystem: Ecosystem::Cargo,
            manifest_kind: None,
        }],
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: None,
    });

    assert!(output.dependencies.is_empty());
    assert!(output.code_lenses.is_empty());
    assert!(output.diagnostics.is_empty());
    assert!(!output.is_supported_manifest);
    assert!(!output.status.visible);
}

#[test]
fn enabled_npm_provider_does_not_enable_pnpm_yaml() {
    let session = session_with_enabled_provider(EnabledProviderConfig {
        ecosystem: Ecosystem::Npm,
        manifest_kind: Some(ManifestKind::NpmPackageJson),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "catalog:\n  react: ^18.0.0\n".to_owned(),
        workspace_root: None,
    });

    assert!(!output.is_supported_manifest);
    assert!(output.dependencies.is_empty());
}

#[test]
fn enabled_pnpm_provider_does_not_enable_package_json() {
    let session = session_with_enabled_provider(EnabledProviderConfig {
        ecosystem: Ecosystem::Npm,
        manifest_kind: Some(ManifestKind::PnpmYaml),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: None,
    });

    assert!(!output.is_supported_manifest);
    assert!(output.dependencies.is_empty());
}

#[test]
fn enabled_deno_provider_keeps_npm_prefixed_deno_imports() {
    let session = session_with_enabled_provider(EnabledProviderConfig {
        ecosystem: Ecosystem::Deno,
        manifest_kind: None,
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: r#"{"imports":{"chalk":"npm:chalk@5.3.0"}}"#.to_owned(),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].ecosystem, "npm");
    assert_eq!(output.dependencies[0].name, "chalk");
}

#[test]
fn configured_file_pattern_classifies_custom_composer_manifest() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ManifestKind::ComposerJson,
                pattern: "**/acme.composer.json".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"require":{"acme/package":"1.2.3"}}"#.to_owned(),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].ecosystem, "composer");
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_brace_alternatives() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ManifestKind::ComposerJson,
                pattern: "**/{composer.json,acme.composer.json}".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"require":{"acme/package":"1.2.3"}}"#.to_owned(),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_workspace_relative_recursive_segments() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ManifestKind::ComposerJson,
                pattern: "packages/**/acme.composer.json".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/packages/backend/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"require":{"acme/package":"1.2.3"}}"#.to_owned(),
        workspace_root: Some("/workspace".to_owned()),
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_character_classes() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ManifestKind::ComposerJson,
                pattern: "**/acme.composer.jso[n]".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"require":{"acme/package":"1.2.3"}}"#.to_owned(),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_character_class_ranges() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ManifestKind::ComposerJson,
                pattern: "**/acme.composer.jso[m-o]".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"require":{"acme/package":"1.2.3"}}"#.to_owned(),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_negated_character_classes() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ManifestKind::ComposerJson,
                pattern: "**/acme.composer.jso[!x]".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"require":{"acme/package":"1.2.3"}}"#.to_owned(),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_file_pattern_supports_micromatch_extglob_alternatives() {
    let session = VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            file_patterns: vec![FilePatternConfig {
                manifest_kind: ManifestKind::ComposerJson,
                pattern: "**/@(composer|acme.composer).json".to_owned(),
            }],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/acme.composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"require":{"acme/package":"1.2.3"}}"#.to_owned(),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "acme/package");
}

#[test]
fn configured_docker_file_pattern_routes_non_yaml_matches_to_dockerfile_parser() {
    let session = session_with_file_pattern(FilePatternConfig {
        manifest_kind: ManifestKind::DockerComposeYaml,
        pattern: "**/Containerfile".to_owned(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/Containerfile".to_owned(),
        language_id: "plaintext".to_owned(),
        text: "FROM node:20\n".to_owned(),
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
        manifest_kind: ManifestKind::PythonRequirementsTxt,
        pattern: "**/pyproject-prod.toml".to_owned(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/pyproject-prod.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: "[project]\ndependencies = [\"requests==2.32.0\"]\n".to_owned(),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].ecosystem, "pypi");
    assert_eq!(output.dependencies[0].group, "project.dependencies");
    assert_eq!(output.dependencies[0].name, "requests");
    assert_eq!(output.dependencies[0].requirement, "==2.32.0");
}

fn session_with_enabled_provider(provider: EnabledProviderConfig) -> VersionLensSession {
    VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![provider],
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    })
}

fn session_with_file_pattern(file_pattern: FilePatternConfig) -> VersionLensSession {
    VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings {
            file_patterns: vec![file_pattern],
            ..ProviderSettings::default()
        },
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    })
}
