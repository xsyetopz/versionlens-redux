use std::env;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;
use std::thread::sleep;

use versionlens_parsers::DocumentInput;

use crate::cache::cache_key;

use crate::{ProviderCacheConfig, ProviderSettings, RegistryResponseInput, SessionConfig};
use versionlens_parsers::Ecosystem::Npm;
use versionlens_parsers::ManifestKind::{NpmPackageJson, PnpmYaml};

#[test]
fn provider_cache_overrides_global_cache_ttl() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            provider_cache: vec![ProviderCacheConfig {
                ecosystem: Npm,
                manifest_kind: None,
                cache_ttl_ms: 1,
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("provider-cache-overrides-global-cache-ttl.json"),
        workspace_root: None,
    };
    let responses = [RegistryResponseInput {
        package: "left-pad".to_owned(),
        ecosystem: Npm,
        body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
    }];

    session.resolve_document_with_responses(input, &responses);
    sleep(crate::duration_from_millis(5));

    assert!(session.cached_latest(&cache_key(Npm, "left-pad")).is_none());
}

#[test]
fn npm_ca_file_context_does_not_write_shared_latest_cache() {
    let root = temp_dir().join(format!("versionlens-npm-cafile-cache-{}", id()));
    create_dir_all(&root).unwrap();
    write(root.join(".npmrc"), "cafile=/tmp/npm-ca.pem\n").unwrap();
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-ca-file-context-does-not-write-shared-latest-cache.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };

    session.resolve_document_with_responses(
        input,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        }],
    );

    assert!(session.cached_latest(&cache_key(Npm, "left-pad")).is_none());
    remove_dir_all(root).unwrap();
}

#[test]
fn manifest_scoped_provider_cache_does_not_override_package_json_npm() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            provider_cache: vec![ProviderCacheConfig {
                ecosystem: Npm,
                manifest_kind: Some(PnpmYaml),
                cache_ttl_ms: 1,
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    assert_eq!(
        session.cache_ttl(Npm, Some(NpmPackageJson)),
        crate::duration_from_millis(300_000)
    );
    assert_eq!(
        session.cache_ttl(Npm, Some(PnpmYaml)),
        crate::duration_from_millis(1)
    );
}

#[test]
fn manifest_scoped_provider_cache_controls_cached_suggestions() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            provider_cache: vec![ProviderCacheConfig {
                ecosystem: Npm,
                manifest_kind: Some(PnpmYaml),
                cache_ttl_ms: 1,
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture(
            "manifest-scoped-provider-cache-controls-cached-suggestions.yaml",
        ),
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
    assert_eq!(
        session.analyze_document(input.clone()).code_lenses[1].title,
        "↑  latest 1.1.0"
    );

    sleep(crate::duration_from_millis(5));

    assert!(session.analyze_document(input).code_lenses.is_empty());
}

#[test]
fn registry_responses_override_cached_latest_version() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("registry-responses-override-cached-latest-version.json"),
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
    let refreshed = session.resolve_document_with_responses(
        input,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"1.2.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(refreshed.edits[0].new_text, "1.2.0");
}

#[test]
fn caches_latest_version_and_clear_cache_removes_it() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("caches-latest-version-and-clear-cache-removes-it.json"),
        workspace_root: None,
    };

    let first = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        }],
    );
    let cached = session.resolve_document_with_responses(input.clone(), &[]);

    session.clear_cache();
    let cleared = session.analyze_document(input);

    assert_eq!(first.edits[0].new_text, "1.1.0");
    assert_eq!(cached.edits[0].new_text, "1.1.0");
    assert!(cleared.diagnostics.is_empty());
}

#[test]
fn cached_latest_preserves_registry_build_aliases() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("cached-latest-preserves-registry-build-aliases.json"),
        workspace_root: None,
    };
    let response = RegistryResponseInput {
        package: "left-pad".to_owned(),
        ecosystem: Npm,
        body: r#"{"dist-tags":{"latest":"1.0.0+build.2"},"versions":{"1.0.0":{},"1.0.0+build.1":{},"1.0.0+build.2":{}}}"#
            .to_owned(),
    };

    let first = session.resolve_document_with_responses(input.clone(), &[response]);
    let cached = session.resolve_document_with_responses(input, &[]);

    assert_eq!(first.suggestions[0].status, "current");
    assert_eq!(cached.suggestions[0].status, "current");
    assert_eq!(cached.suggestions[0].builds, first.suggestions[0].builds);
}

#[test]
fn clear_cache_removes_dotnet_registry_sources() {
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

    *session.dotnet_registry_sources.lock().unwrap() =
        Some(vec!["https://nuget.test/v3/index.json".to_owned()]);

    session.clear_cache();

    assert!(session.dotnet_registry_sources.lock().unwrap().is_none());
}

#[test]
fn analyze_document_uses_cached_latest_for_code_lens_title() {
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
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("analyze-document-uses-cached-latest-for-code-lens-title.json"),
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

    assert_eq!(output.code_lenses[0].title, "🟡 fixed 1.0.0");
    assert_eq!(output.code_lenses[0].command, "");
    assert_eq!(output.code_lenses[1].title, "↑  latest 1.1.0");
    assert_eq!(
        output.code_lenses[1].command,
        "versionlens.suggestion.onUpdateDependency"
    );
    assert_eq!(output.code_lenses[1].arguments[0], "left-pad");
    assert!(output.code_lenses[1].arguments[1].starts_with("left-pad"));
}

#[test]
fn cached_latest_is_scoped_to_dependency_requirement_for_update_choices() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let fixed_input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "cached-latest-is-scoped-to-dependency-requirement-for-update-choices.json",
        ),
        workspace_root: None,
    };
    let range_input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "cached-latest-is-scoped-to-dependency-requirement-for-update-choices-2.json",
        ),
        workspace_root: None,
    };

    session.resolve_document_with_responses(
        fixed_input,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "2.0.0" },
              "versions": {
                "1.0.0": {},
                "1.1.1": {},
                "1.1.2": {},
                "2.0.0": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let cached_range = session.resolve_document_with_responses(range_input.clone(), &[]);
    let code_lenses = session.analyze_document(range_input).code_lenses;
    let titles = code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = code_lenses
        .iter()
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(cached_range.suggestions[0].status, "satisfies");
    assert_eq!(
        titles,
        ["🟡 satisfies 1.1.2", "↑  latest 2.0.0", "↑  bump 1.1.2"]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "2.0.0"],
            vec!["update", "1.1.2"]
        ]
    );
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root().join("tests/fixtures/session/cache").join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session cache fixture {}: {error}",
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
