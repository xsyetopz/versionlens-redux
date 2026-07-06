use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::DocumentInput;

use super::resolve_worker_count;

use crate::{RegistryResponseInput, SessionConfig};
use versionlens_parsers::Ecosystem::Npm;

#[test]
fn batched_resolution_preserves_dependency_order() {
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
    let names = [
        "dep-00", "dep-01", "dep-02", "dep-03", "dep-04", "dep-05", "dep-06", "dep-07", "dep-08",
        "dep-09", "dep-10", "dep-11",
    ];
    let responses = names
        .iter()
        .map(|name| RegistryResponseInput {
            package: (*name).to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        })
        .collect::<Vec<_>>();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("batched-resolution-preserves-dependency-order.json"),
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
    assert_eq!(resolve_worker_count(0), 0);
    assert_eq!(resolve_worker_count(1), 1);
    assert_eq!(resolve_worker_count(8), 8);
    assert_eq!(resolve_worker_count(12), 8);
    assert_eq!(resolve_worker_count(100), 8);
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/parallel/tests")
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
