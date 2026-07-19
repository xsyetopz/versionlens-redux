use versionlens_model::DocumentInput;

use crate::{RegistryResponseInput, SessionConfig};

use super::{package_file_fixture, test_indicators};
use versionlens_model::Ecosystem::Npm;

#[test]
fn code_lens_title_marks_vulnerable_update_targets() {
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

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "1.1.0" },
              "vulns": [{
                "id": "OSV-1",
                "summary": "target issue",
                "affected": [{
                  "package": { "name": "left-pad" },
                  "ranges": [{
                    "events": [{ "introduced": "1.1.0" }, { "fixed": "2.0.0" }]
                  }]
                }]
              }]
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[1].title, "V latest 1.1.0");
}

#[test]
fn code_lens_title_does_not_mark_update_that_fixes_current_vulnerability() {
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

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "1.1.0" },
              "vulns": [{
                "id": "OSV-1",
                "summary": "current issue",
                "affected": [{
                  "package": { "name": "left-pad" },
                  "ranges": [{
                    "events": [{ "introduced": "0" }, { "fixed": "1.1.0" }]
                  }]
                }]
              }]
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[1].title, "U latest 1.1.0");
}

#[test]
fn vulnerable_update_indicator_falls_back_to_warning_when_configured_indicator_is_empty() {
    let mut indicators = test_indicators();
    indicators.updateable_vulnerable = "".to_owned();
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
        text: package_file_fixture("package-left-pad-1.1.1.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "versions": {
                "1.1.1": {},
                "1.1.2": {},
                "1.2.2": {},
                "2.2.2": {}
              },
              "vulns": [{
                "id": "OSV-MINOR",
                "summary": "minor target issue",
                "affected": [{
                  "package": { "name": "left-pad" },
                  "ranges": [{
                    "events": [{ "introduced": "1.2.2" }, { "fixed": "1.2.3" }]
                  }]
                }]
              }]
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();

    assert!(titles.contains(&"⚠️ minor 1.2.2"));
    assert!(!titles.contains(&"U minor 1.2.2"));
}

#[test]
fn vulnerable_build_code_lens_uses_vulnerable_update_indicator_fallback() {
    let mut indicators = test_indicators();
    indicators.updateable_vulnerable = "".to_owned();
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
        text: package_file_fixture("package-left-pad-1.0.0-b1.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "1.0.0+b2" },
              "versions": {
                "1.0.0+b1": {},
                "1.0.0+b2": {}
              },
              "vulns": [{
                "id": "OSV-BUILD",
                "summary": "build target issue",
                "affected": [{
                  "package": { "name": "left-pad" },
                  "versions": ["1.0.0+b1"]
                }]
              }]
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();

    assert!(titles.contains(&"⚠️ change build"));
    assert!(!titles.contains(&"B change build"));
}

#[test]
fn update_choice_code_lens_marks_vulnerable_non_latest_targets() {
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
        text: package_file_fixture("package-left-pad-1.1.1.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "versions": {
                "1.1.1": {},
                "1.1.2": {},
                "1.2.2": {},
                "2.2.2": {}
              },
              "vulns": [{
                "id": "OSV-MINOR",
                "summary": "minor target issue",
                "affected": [{
                  "package": { "name": "left-pad" },
                  "ranges": [{
                    "events": [{ "introduced": "1.2.2" }, { "fixed": "1.2.3" }]
                  }]
                }]
              }]
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        titles,
        [
            "M fixed 1.1.1",
            "U latest 2.2.2",
            "V minor 1.2.2",
            "U patch 1.1.2"
        ]
    );
}
