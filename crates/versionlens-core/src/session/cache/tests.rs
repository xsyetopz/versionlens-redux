use crate::RegistryResponseInput;
use crate::{ProviderCacheConfig, ProviderSettings, SessionConfig};
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::write;
use std::process::id;
use std::thread::sleep;

use versionlens_model::DocumentInput;

use crate::cache::cache_key;

use versionlens_http::HttpHeader;
use versionlens_model::Ecosystem::Npm;
use versionlens_model::ManifestKind::{NpmPackageJson, PnpmYaml};

fn session_with_manifest_cache(
    manifest_kind: Option<versionlens_model::ManifestKind>,
) -> crate::VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            provider_cache: vec![ProviderCacheConfig {
                ecosystem: Npm,
                manifest_kind,
                cache_ttl_ms: 1,
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}

#[test]
fn request_cache_identity_separates_effective_security_context_without_exposing_secrets() {
    let session = crate::support::tests::test_session(false);
    let mut first = versionlens_http::standard_http_config();
    first.auth_headers.push(HttpHeader {
        name: "authorization".to_owned(),
        value: "Bearer first-secret".to_owned(),
        url: None,
    });
    let mut second = first.clone();
    second.auth_headers[0].value = "Bearer second-secret".to_owned();
    let mut proxied = first.clone();
    proxied.proxy = Some("http://proxy.example.test".to_owned());
    let mut custom_tls = first.clone();
    custom_tls.strict_ssl = false;
    custom_tls.ca = Some("private-ca-material".to_owned());

    let first_key = session.request_cache_key("https://registry.example.test/pkg", &first);
    let second_key = session.request_cache_key("https://registry.example.test/pkg", &second);
    let proxied_key = session.request_cache_key("https://registry.example.test/pkg", &proxied);
    let custom_tls_key =
        session.request_cache_key("https://registry.example.test/pkg", &custom_tls);

    assert_ne!(first_key, second_key);
    assert_ne!(first_key, proxied_key);
    assert_ne!(first_key, custom_tls_key);
    assert!(!first_key.as_str().contains("first-secret"));
    assert!(!second_key.as_str().contains("second-secret"));
    assert!(!custom_tls_key.as_str().contains("private-ca-material"));

    session.cache_request_body(first_key.clone(), "first response", Npm, None);
    assert_eq!(
        session.cached_request_body(&first_key).as_deref(),
        Some("first response")
    );
    assert!(session.cached_request_body(&second_key).is_none());
    assert!(session.cached_request_body(&proxied_key).is_none());
    assert!(session.cached_request_body(&custom_tls_key).is_none());
}

#[test]
fn completed_request_lock_keys_are_pruned_during_subsequent_requests() {
    let session = crate::support::tests::test_session(false);
    let http = versionlens_http::standard_http_config();

    for index in 0..100 {
        let key = session.request_cache_key(
            &format!("https://registry.example.test/package-{index}"),
            &http,
        );
        drop(session.request_lock(&key));
    }

    assert_eq!(session.request_locks.lock().unwrap().len(), 1);
}

#[test]
fn provider_cache_overrides_global_cache_ttl() {
    let session = session_with_manifest_cache(None);

    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("provider-cache-overrides-global-cache-ttl.json"),
        None,
    );
    let responses = [RegistryResponseInput::new(
        "left-pad".to_owned(),
        Npm,
        r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
    )];

    session.resolve_document_with_responses(input, &responses);
    sleep(crate::duration_from_millis(5));

    assert!(session.cached_latest(&cache_key(Npm, "left-pad")).is_none());
}

#[test]
fn npm_ca_file_context_does_not_write_shared_latest_cache() {
    let root = temp_dir().join(format!("versionlens-npm-cafile-cache-{}", id()));
    create_dir_all(&root).unwrap();
    write(root.join(".npmrc"), "cafile=/tmp/npm-ca.pem\n").unwrap();
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture("npm-ca-file-context-does-not-write-shared-latest-cache.txt"),
        Some(root.to_string_lossy().into_owned()),
    );

    session.resolve_document_with_responses(
        input,
        &[crate::support::tests::npm_latest_response(
            "left-pad", "1.1.0",
        )],
    );

    assert!(session.cached_latest(&cache_key(Npm, "left-pad")).is_none());
    remove_dir_all(root).unwrap();
}

#[test]
fn manifest_scoped_provider_cache_does_not_override_package_json_npm() {
    let session = session_with_manifest_cache(Some(PnpmYaml));

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
    let session = session_with_manifest_cache(Some(PnpmYaml));
    let input = DocumentInput::new(
        "file:///pnpm-workspace.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("manifest-scoped-provider-cache-controls-cached-suggestions.yaml"),
        None,
    );

    crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[crate::support::tests::npm_latest_response(
            "left-pad", "1.1.0",
        )],
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
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("registry-responses-override-cached-latest-version.json"),
        None,
    );

    session.resolve_document_with_responses(
        input.clone(),
        &[crate::support::tests::npm_latest_response(
            "left-pad", "1.1.0",
        )],
    );
    let refreshed = session.resolve_document_with_responses(
        input,
        &[crate::support::tests::npm_latest_response(
            "left-pad", "1.2.0",
        )],
    );

    assert_eq!(refreshed.edits[0].new_text, "1.2.0");
}

#[test]
fn caches_latest_version_and_clear_cache_removes_it() {
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("caches-latest-version-and-clear-cache-removes-it.json"),
        None,
    );

    let first = session.resolve_document_with_responses(
        input.clone(),
        &[crate::support::tests::npm_latest_response(
            "left-pad", "1.1.0",
        )],
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
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("latest-preserves-registry-build-aliases.json"),
        None,
    );
    let response = RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{"dist-tags":{"latest":"1.0.0+build.2"},"versions":{"1.0.0":{},"1.0.0+build.1":{},"1.0.0+build.2":{}}}"#
            .to_owned());

    let first = session.resolve_document_with_responses(input.clone(), &[response]);
    let cached = session.resolve_document_with_responses(input, &[]);

    assert_eq!(first.suggestions[0].status, "current");
    assert_eq!(cached.suggestions[0].status, "current");
    assert_eq!(cached.suggestions[0].builds, first.suggestions[0].builds);
}

#[test]
fn clear_cache_removes_dotnet_registry_sources() {
    let session = crate::support::tests::test_session(true);

    *session.dotnet_registry_sources.lock().unwrap() =
        Some(vec!["https://nuget.test/v3/index.json".to_owned()]);

    session.clear_cache();

    assert!(session.dotnet_registry_sources.lock().unwrap().is_none());
}

#[test]
fn analyze_document_uses_cached_latest_for_code_lens_title() {
    let session = crate::support::tests::test_session(true);
    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("analyze-document-uses-cached-latest-for-code-lens-title.json"),
        None,
    );

    session.resolve_document_with_responses(
        input.clone(),
        &[crate::support::tests::npm_latest_response(
            "left-pad", "1.1.0",
        )],
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
    let session = crate::support::tests::test_session(false);
    let fixed_input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("latest-is-scoped-to-dependency-requirement-for-update-choices.json"),
        None,
    );
    let range_input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("latest-is-scoped-to-dependency-requirement-for-range.json"),
        None,
    );

    session.resolve_document_with_responses(
        fixed_input,
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{
              "dist-tags": { "latest": "2.0.0" },
              "versions": {
                "1.0.0": {},
                "1.1.1": {},
                "1.1.2": {},
                "2.0.0": {}
              }
            }"#
            .to_owned(),
        )],
    );

    let cached_range = session.resolve_document_with_responses(range_input.clone(), &[]);
    let analysis = session.analyze_document(range_input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = crate::support::tests::all_code_lens_arguments(&analysis);

    assert_eq!(cached_range.suggestions[0].status, "satisfies");
    assert_eq!(
        titles,
        ["🟡 satisfies 1.1.2", "↑  bump 1.1.2", "↑  latest 2.0.0"]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "1.1.2"],
            vec!["update", "2.0.0"]
        ]
    );
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/cache", name)
}
