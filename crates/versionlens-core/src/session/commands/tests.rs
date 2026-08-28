use super::ApplyCommandRequest;
use versionlens_model::{DocumentInput, Ecosystem};

use crate::{
    DependencyPropertyConfig, ProviderSettings, RegistryResponseInput, SessionConfig,
    VersionLensSession,
};

fn standard_session() -> VersionLensSession {
    crate::support::tests::test_session(true)
}

fn session_with_vulnerability_visibility(show_vulnerabilities: bool) -> VersionLensSession {
    let mut session = standard_session();
    session.config.show_vulnerabilities = show_vulnerabilities;
    session
}

fn session_with_dependency_properties(
    ecosystem: Ecosystem,
    properties: &[&str],
) -> VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            dependency_properties: vec![DependencyPropertyConfig {
                ecosystem,
                manifest_kind: None,
                properties: properties
                    .iter()
                    .map(|property| (*property).to_owned())
                    .collect(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}

fn assert_single_named_edit(
    output: &crate::contract::ResolveDocumentOutput,
    name: &str,
    new_text: &str,
) {
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, name);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, new_text);
}

mod command_contracts;
mod sort;
mod update;
