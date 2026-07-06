use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::DocumentInput;

use crate::{PrereleaseTagConfig, ProviderSettings, RegistryResponseInput, SessionConfig};
use versionlens_parsers::Ecosystem::{Composer, Dotnet, Npm, Python};

#[test]
fn show_prereleases_allows_prerelease_updates() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: versionlens_http::standard_http_config(),
    });

    let input = DocumentInput {
        uri: "file:///Directory.Packages.props".to_owned(),
        language_id: "xml".to_owned(),
        text: package_file_fixture("show-prereleases-allows-prerelease-updates.Packages.props"),
        workspace_root: None,
    };
    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "Newtonsoft.Json".to_owned(),
            ecosystem: Dotnet,
            body: r#"{"versions":["13.0.3","14.0.0-beta.1"]}"#.to_owned(),
        }],
    );
    let analysis = session.analyze_document(input);
    let update_arguments = analysis
        .code_lenses
        .iter()
        .filter(|lens| lens.title == "↑  beta 14.0.0-beta.1")
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
    assert_eq!(update_arguments, [vec!["update", "14.0.0-beta.1"]]);
}

#[test]
fn show_prereleases_applies_to_composer_update_choices() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: versionlens_http::standard_http_config(),
    });

    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("show-prereleases-applies-to-composer-update-choices.json"),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Composer,
            body: r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v3.1.3" },
                  { "version": "v3.2.0-beta.1" }
                ]
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);

    assert!(
        analysis
            .code_lenses
            .iter()
            .any(|lens| lens.title == "↑  beta 3.2.0-beta.1")
    );
}

#[test]
fn show_prereleases_applies_to_npm_versions() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: versionlens_http::standard_http_config(),
    });

    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("show-prereleases-applies-to-npm-versions.json"),
        workspace_root: None,
    };
    let responses = [RegistryResponseInput {
        package: "typescript".to_owned(),
        ecosystem: Npm,
        body: r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#
            .to_owned(),
    }];

    session.resolve_document_with_responses(input.clone(), &responses);
    let output = session.analyze_document(input);
    let arguments = output
        .code_lenses
        .iter()
        .filter_map(|lens| {
            if lens.command != "versionlens.suggestion.onUpdateDependency" {
                return None;
            }
            let command = lens.arguments.get(2)?;
            let version = lens.arguments.get(3)?;
            Some(vec![command.as_str(), version.as_str()])
        })
        .collect::<Vec<_>>();

    assert_eq!(
        arguments,
        [vec!["update", "6.0.3"], vec!["update", "7.0.0-beta.1"]]
    );
}

#[test]
fn show_prereleases_keeps_npm_prerelease_choice_when_fixed_version_is_latest() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: versionlens_http::standard_http_config(),
    });

    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "show-prereleases-keeps-npm-prerelease-choice-when-fixed-version-is-latest.json",
        ),
        workspace_root: None,
    };
    let responses = [RegistryResponseInput {
        package: "left-pad".to_owned(),
        ecosystem: Npm,
        body: r#"{
          "dist-tags": { "latest": "3.0.0" },
          "versions": {
            "1.0.0": {},
            "1.1.0-alpha.1": {},
            "2.0.0": {},
            "2.1.0": {},
            "3.0.0": {},
            "4.0.0-next": {}
          }
        }"#
        .to_owned(),
    }];

    let resolved = session.resolve_document_with_responses(input.clone(), &responses);
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

    assert_eq!(resolved.suggestions[0].status, "current");
    assert_eq!(resolved.suggestions[0].latest.as_deref(), Some("3.0.0"));
    assert!(resolved.edits.is_empty());
    assert_eq!(titles, ["🟢 latest 3.0.0", "↑  next 4.0.0-next"]);
    assert_eq!(arguments, [vec!["update", "4.0.0-next"]]);
}

#[test]
fn prerelease_tag_filters_apply_to_responses() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            prerelease_tags: vec![PrereleaseTagConfig {
                ecosystem: Npm,
                tags: vec!["beta".to_owned()],
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: versionlens_http::standard_http_config(),
    });

    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("prerelease-tag-filters-apply-to-responses.json"),
        workspace_root: None,
    };
    let responses = [RegistryResponseInput {
        package: "typescript".to_owned(),
        ecosystem: Npm,
        body: r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{},"8.0.0-rc.1":{}}}"#
            .to_owned(),
    }];

    session.resolve_document_with_responses(input.clone(), &responses);
    let output = session.analyze_document(input);
    let arguments = output
        .code_lenses
        .iter()
        .filter_map(|lens| {
            if lens.command != "versionlens.suggestion.onUpdateDependency" {
                return None;
            }
            let command = lens.arguments.get(2)?;
            let version = lens.arguments.get(3)?;
            Some(vec![command.as_str(), version.as_str()])
        })
        .collect::<Vec<_>>();

    assert_eq!(
        arguments,
        [vec!["update", "6.0.3"], vec!["update", "7.0.0-beta.1"]]
    );
}

#[test]
fn prerelease_ranges_can_resolve_prerelease_versions_when_hidden() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "prerelease-ranges-can-resolve-prerelease-versions-when-hidden.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "typescript".to_owned(),
            ecosystem: Npm,
            body: r#"{"versions":{"2.0.0-beta.1":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits[0].new_text, "^2.0.0-beta.1");
}

#[test]
fn show_prereleases_applies_to_python_releases() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: true,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: package_file_fixture("show-prereleases-applies-to-python-releases.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "flask".to_owned(),
            ecosystem: Python,
            body: r#"{"info":{"version":"3.0.0"},"releases":{"3.0.0":[],"4.0.0rc1":[]}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.edits[0].new_text, "==4.0.0rc1");
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/core/prerelease/tests")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
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
