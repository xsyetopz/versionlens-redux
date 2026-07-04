use versionlens_http::HttpConfig;
use versionlens_parsers::DocumentInput;

use crate::{ProviderSettings, SessionConfig, SuggestionIndicators, VersionLensSession};

fn session() -> VersionLensSession {
    VersionLensSession::new(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: Vec::new(),
        providers: ProviderSettings::default(),
        suggestion_indicators: SuggestionIndicators::standard(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: HttpConfig::standard(),
    })
}

#[test]
fn analyzes_extension_schema_documents_without_dependency_diagnostics() {
    let valid = session().analyze_document(DocumentInput {
        uri: "versionlens:/versionlens.multi-registries.json".to_owned(),
        language_id: "json".to_owned(),
        text: "{}".to_owned(),
        workspace_root: None,
    });

    assert!(valid.is_supported_manifest);
    assert!(valid.diagnostics.is_empty());
    assert!(valid.dependencies.is_empty());
    assert!(!valid.can_sort_dependencies);

    let invalid = session().analyze_document(DocumentInput {
        uri: "versionlens:/versionlens.multi-registries.json".to_owned(),
        language_id: "json".to_owned(),
        text: "not json".to_owned(),
        workspace_root: None,
    });

    assert!(invalid.is_supported_manifest);
    assert!(invalid.diagnostics.is_empty());
}
