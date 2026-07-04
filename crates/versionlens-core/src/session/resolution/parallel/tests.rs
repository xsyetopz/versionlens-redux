use versionlens_http::HttpConfig;
use versionlens_parsers::{DocumentInput, Ecosystem};

use crate::{
    ProviderSettings, RegistryResponseInput, SessionConfig, SuggestionIndicators,
    VersionLensSession,
};

#[test]
fn batched_resolution_preserves_dependency_order() {
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
    let names = [
        "dep-00", "dep-01", "dep-02", "dep-03", "dep-04", "dep-05", "dep-06", "dep-07", "dep-08",
        "dep-09", "dep-10", "dep-11",
    ];
    let responses = names
        .iter()
        .map(|name| RegistryResponseInput {
            package: (*name).to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        })
        .collect::<Vec<_>>();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"dep-00":"1.0.0","dep-01":"1.0.0","dep-02":"1.0.0","dep-03":"1.0.0","dep-04":"1.0.0","dep-05":"1.0.0","dep-06":"1.0.0","dep-07":"1.0.0","dep-08":"1.0.0","dep-09":"1.0.0","dep-10":"1.0.0","dep-11":"1.0.0"}}"#
                .to_owned(),
            workspace_root: None,
        },
        &responses,
    );
    let resolved_names = output
        .suggestions
        .iter()
        .map(|suggestion| suggestion.dependency.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(resolved_names, names);
}

#[test]
fn registry_resolution_bounds_parallel_workers() {
    assert_eq!(super::resolve_worker_count(0), 0);
    assert_eq!(super::resolve_worker_count(1), 1);
    assert_eq!(super::resolve_worker_count(8), 8);
    assert_eq!(super::resolve_worker_count(12), 8);
    assert_eq!(super::resolve_worker_count(100), 8);
}
