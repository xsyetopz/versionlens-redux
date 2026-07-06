use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::DocumentInput;

use crate::{SessionConfig, VersionLensSession};

fn session() -> VersionLensSession {
    crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    })
}

#[test]
fn analyzes_extension_schema_documents_without_dependency_diagnostics() {
    let valid = session().analyze_document(DocumentInput {
        uri: "versionlens:/versionlens.multi-registries.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("analyzes-extension-schema-documents-without-dependency-diagnostics.multi-registries.json"),
        workspace_root: None,
    });

    assert!(valid.is_supported_manifest);
    assert!(valid.diagnostics.is_empty());
    assert!(valid.dependencies.is_empty());
    assert!(!valid.can_sort_dependencies);

    let invalid = session().analyze_document(DocumentInput {
        uri: "versionlens:/versionlens.multi-registries.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("analyzes-extension-schema-documents-without-dependency-diagnostics.multi-registries-2.multi-registries.json"),
        workspace_root: None,
    });

    assert!(invalid.is_supported_manifest);
    assert!(invalid.diagnostics.is_empty());
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/core/schema/tests")
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
