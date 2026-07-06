use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::DocumentInput;

use crate::{RegistryResponseInput, SessionConfig};
use versionlens_parsers::Ecosystem::Dub;

#[test]
fn resolves_repo_versions_as_current_when_registry_matches() {
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
            uri: "file:///dub.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-repo-versions-as-current-when-registry-matches.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "vibe-d".to_owned(),
            ecosystem: Dub,
            body: r#"{"versions":[{"version":"~master"},{"version":"0.9.0"},{"version":"0.8.0"}]}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].latest.as_deref(), Some("~master"));
    assert_eq!(output.suggestions[0].status, "current");
    assert!(output.edits.is_empty());
}

#[test]
fn pub_hosted_git_dependencies_are_fixed_without_git_suffix() {
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
    let output = session.resolve_document(DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pub-hosted-git-dependencies-are-fixed-without-git-suffix.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.suggestions[0].dependency.name, "repo");
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
}

#[test]
fn pub_git_tag_pattern_dependencies_are_fixed_without_pub_registry_lookup() {
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
    let output = session.resolve_document(DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture(
            "pub-git-tag-pattern-dependencies-are-fixed-without-pub-registry-lookup.yaml",
        ),
        workspace_root: None,
    });

    assert_eq!(output.suggestions[0].dependency.name, "kittens");
    assert_eq!(
        output.suggestions[0].dependency.requirement,
        "git@github.com:munificent/kittens.git"
    );
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert!(output.edits.is_empty());
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/dependency/tests")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
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
