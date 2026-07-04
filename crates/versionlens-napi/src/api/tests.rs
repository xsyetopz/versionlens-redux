use crate::model::{NativeDocumentInput, NativeSessionConfig};

use super::create_session;

fn session_config() -> NativeSessionConfig {
    NativeSessionConfig {
        cache_duration_minutes: None,
        enabled_providers: None,
        providers: None,
        suggestion_indicators: None,
        show_vulnerabilities: None,
        show_suggestion_stats: None,
        show_prereleases: false,
        http: None,
    }
}

fn package_document() -> NativeDocumentInput {
    NativeDocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: None,
    }
}

#[test]
fn dispose_session_releases_inner_session() {
    let session = create_session(session_config());
    let output = session.analyze_document(package_document());
    assert!(output.is_supported_manifest);
    assert_eq!(output.active_provider_name, Some("npm".to_owned()));

    session.dispose_session();
    session.clear_cache();

    let output = session.analyze_document(package_document());
    assert!(!output.is_supported_manifest);
    assert!(!output.status.visible);
}
