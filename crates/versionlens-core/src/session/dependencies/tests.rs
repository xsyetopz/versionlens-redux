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
