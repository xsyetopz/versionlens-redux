use std::fs::read_to_string;
use std::path::PathBuf;

use versionlens_parsers::DocumentInput;

use crate::{RegistryResponseInput, SessionConfig, SuggestionIndicators};
use versionlens_parsers::Ecosystem::Npm;

mod actions;
mod docker;
mod npm;
mod ranges;
mod vulnerabilities;
#[test]
fn code_lens_title_uses_configured_indicators() {
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
            body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        }],
    );

    let output = session.analyze_document(input);

    let titles = output
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let commands = output
        .code_lenses
        .iter()
        .map(|lens| lens.command.as_str())
        .collect::<Vec<_>>();

    assert_eq!(titles, ["M fixed 1.0.0", "U latest 1.1.0"]);
    assert_eq!(commands, ["", "versionlens.suggestion.onUpdateDependency"]);
}

#[test]
fn code_lenses_offer_release_update_choices_for_fixed_versions() {
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
              "dist-tags": { "latest": "2.1.0" },
              "versions": {
                "1.0.0": {},
                "1.0.1": {},
                "1.1.0": {},
                "2.0.0": {},
                "2.1.0": {}
              }
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
    let arguments = output
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        titles,
        [
            "M fixed 1.0.0",
            "U latest 2.1.0",
            "U minor 1.1.0",
            "U patch 1.0.1"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "2.1.0"],
            vec!["updateMinor", "1.1.0"],
            vec!["updatePatch", "1.0.1"]
        ]
    );
}

#[test]
fn code_lens_ranges_encode_suggestion_order() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: false,
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
              "dist-tags": { "latest": "2.1.0" },
              "versions": {
                "1.0.0": {},
                "1.0.1": {},
                "1.1.0": {},
                "2.1.0": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let output = session.analyze_document(input);
    let starts = output
        .code_lenses
        .iter()
        .map(|lens| lens.range.start.character)
        .collect::<Vec<_>>();
    let zero_width = output
        .code_lenses
        .iter()
        .all(|lens| lens.range.start == lens.range.end);

    assert_eq!(starts, [18, 19, 20, 21]);
    assert!(zero_width);
}

#[test]
fn multiline_package_json_code_lenses_stay_on_dependency_lines() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: test_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-dev-dependencies.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[
            npm_response("@biomejs/biome", "2.5.2"),
            npm_response("@types/bun", "1.3.14"),
            npm_response("@types/node", "26.0.1"),
            npm_response("@types/vscode", "1.125.0"),
            npm_response("@vscode/vsce", "3.9.2"),
            npm_response("typescript", "6.0.3"),
        ],
    );

    let output = session.analyze_document(input);
    let lenses = output
        .code_lenses
        .iter()
        .map(|lens| {
            (
                lens.range.start.line,
                lens.title.as_str(),
                lens.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        lenses,
        [
            (2, "L latest 2.5.2", ""),
            (3, "L latest 1.3.14", ""),
            (4, "L latest 26.0.1", ""),
            (5, "S satisfies latest 1.125.0", ""),
            (6, "L latest 3.9.2", ""),
            (7, "L latest 6.0.3", "")
        ]
    );
}

#[test]
fn code_lenses_offer_bump_update_choices_for_ranges() {
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
        text: package_file_fixture("package-left-pad-4.1.0-range.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "5.4.5" },
              "versions": {
                "2.1.2": {},
                "3.0.0": {},
                "3.1.0": {},
                "4.0.0": {},
                "4.0.1": {},
                "4.1.10": {},
                "5.1.1": {},
                "5.2.0": {},
                "5.3.3": {},
                "5.4.5": {}
              }
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
    let arguments = output
        .code_lenses
        .iter()
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        titles,
        ["M satisfies 4.1.10", "U latest 5.4.5", "U bump 4.1.10"]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "5.4.5"],
            vec!["update", "4.1.10"]
        ]
    );
}

#[test]
fn code_lenses_keep_latest_update_choice_for_invalid_ranges() {
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
        text: package_file_fixture("package-left-pad-invalid-range.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "5.0.0" },
              "versions": {
                "1.0.0": {},
                "2.0.0": {},
                "5.0.0": {}
              }
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
    let arguments = output
        .code_lenses
        .iter()
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(titles, ["E invalid version range", "U latest 5.0.0"]);
    assert_eq!(arguments, [Vec::<&str>::new(), vec!["update", "5.0.0"]]);
}

#[test]
fn code_lenses_keep_latest_update_choice_for_ranges_satisfying_latest() {
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
        text: package_file_fixture("package-left-pad-gte-2.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "3.0.0" },
              "versions": {
                "1.0.0": {},
                "2.0.0": {},
                "2.1.0": {},
                "3.0.0": {}
              }
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
    let arguments = output
        .code_lenses
        .iter()
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(titles, ["S satisfies latest 3.0.0", "U latest 3.0.0"]);
    assert_eq!(arguments, [Vec::<&str>::new(), vec!["update", "3.0.0"]]);
}

#[test]
fn code_lenses_keep_satisfies_status_for_ranges_with_in_range_updates() {
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
        text: package_file_fixture("package-left-pad-gte-2-lt-3.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "3.0.0" },
              "versions": {
                "1.0.0": {},
                "2.0.0": {},
                "2.1.0": {},
                "3.0.0": {}
              }
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
    let arguments = output
        .code_lenses
        .iter()
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        titles,
        ["M satisfies 2.1.0", "U latest 3.0.0", "U bump 2.1.0"]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "3.0.0"],
            vec!["update", "2.1.0"]
        ]
    );
}

#[test]
fn code_lenses_offer_prerelease_update_choices_by_tag() {
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
        text: package_file_fixture("package-left-pad-prerelease-range.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "versions": {
                "1.0.0-alpha": {},
                "1.0.1-alpha": {},
                "1.2.0-alpha": {},
                "1.2.0-dev": {},
                "1.2.0-beta": {}
              }
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
            "U dev 1.2.0-dev",
            "U beta 1.2.0-beta",
            "U alpha 1.2.0-alpha"
        ]
    );
}

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

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}

fn npm_response(package: &str, latest: &str) -> RegistryResponseInput {
    RegistryResponseInput {
        package: package.to_owned(),
        ecosystem: Npm,
        body: format!(r#"{{"dist-tags":{{"latest":"{latest}"}}}}"#),
    }
}

fn test_indicators() -> SuggestionIndicators {
    SuggestionIndicators {
        latest: "L".to_owned(),
        satisfies_latest: "S".to_owned(),
        directory: "D".to_owned(),
        error: "E".to_owned(),
        no_match: "N".to_owned(),
        matched: "M".to_owned(),
        updateable: "U".to_owned(),
        updateable_vulnerable: "V".to_owned(),
        build: "B".to_owned(),
    }
}
