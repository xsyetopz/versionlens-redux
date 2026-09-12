use versionlens_model::DocumentInput;

use crate::{
    EnabledProviderConfig, FilePatternConfig, ProviderSettings, SessionConfig, VersionLensSession,
};
use versionlens_model::Ecosystem::*;
use versionlens_model::ManifestKind::{
    Cabal, ComposerJson, DenoJson, DockerComposeYaml, DubJson, Gemfile, GleamToml,
    JuliaProjectToml, MixExs, NpmPackageJson, Opam, PnpmYaml, PythonRequirementsTxt, RDescription,
    RebarConfig,
};

include!("tests/providers.rs");
include!("tests/patterns.rs");

#[test]
fn changed_snapshot_reconstructs_requirements_and_unicode_ranges() {
    let session = crate::support::tests::test_session(false);
    let initial = DocumentInput::new(
        "file:///package.json",
        "json",
        r#"{"dependencies":{"example":"1.0.0"}}"#,
        None,
    );
    let first = session.dependencies(&initial);
    let mut changed = initial.clone();
    changed.text =
        "{\n  \"description\": \"🦀\", \"dependencies\": {\"example\": \"2.0.0\"}\n}".to_owned();
    let second = session.dependencies(&changed);
    assert_eq!(first[0].requirement, "1.0.0");
    assert_eq!(second[0].requirement, "2.0.0");
    assert_eq!(second[0].requirement_range.start.line, 1);
    assert_ne!(first[0].requirement_range, second[0].requirement_range);
    assert_eq!(session.dependencies(&initial), first);
    assert_eq!(session.dependencies(&changed), second);
}

#[test]
fn configured_patterns_match_decoded_workspace_paths() {
    let session = session_with_file_pattern(FilePatternConfig {
        manifest_kind: NpmPackageJson,
        pattern: "ci files/dépendances.custom".to_owned(),
    });
    for root in [
        "/workspace/project files",
        "file:///workspace/project%20files",
    ] {
        let input = DocumentInput {
            uri: "file:///workspace/project%20files/ci%20files/d%C3%A9pendances.custom".to_owned(),
            language_id: "plaintext".to_owned(),
            text: r#"{"dependencies":{"example":"1.0.0"}}"#.to_owned(),
            workspace_root: Some(root.to_owned()),
            version: None,
        };
        let analysis = session.analyze_document(input);
        assert!(analysis.is_supported_manifest);
        assert_eq!(analysis.dependencies.len(), 1);
        assert_eq!(analysis.dependencies[0].name, "example");
    }
}
fn session_with_enabled_provider(provider: EnabledProviderConfig) -> VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![provider],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}

fn session_with_file_pattern(file_pattern: FilePatternConfig) -> VersionLensSession {
    crate::version_lens_session(crate::support::tests::session_config(
        ProviderSettings {
            file_patterns: vec![file_pattern],
            ..crate::default()
        },
        true,
    ))
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/dependencies", name)
}
