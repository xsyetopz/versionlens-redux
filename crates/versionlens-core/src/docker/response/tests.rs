use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::DocumentInput;

use crate::{RegistryResponseInput, SessionConfig};
use versionlens_parsers::Ecosystem::Docker;

#[test]
fn docker_registry_response_missing_requested_tag_creates_no_match() {
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
            uri: "file:///Dockerfile".to_owned(),
            language_id: "dockerfile".to_owned(),
            text: package_file_fixture(
                "docker-registry-response-missing-requested-tag-creates-no-matchDockerfile",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "node".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"2.0.0","tag_status":"active","digest":"sha256-2"}]}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
}

#[test]
fn docker_same_digest_aliases_keep_current_status_and_create_build_suggestions() {
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
            uri: "file:///Dockerfile".to_owned(),
            language_id: "dockerfile".to_owned(),
            text: package_file_fixture("docker-same-digest-aliases-keep-current-status-and-create-build-suggestionsDockerfile"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "node".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"current-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"current","tag_status":"active","digest":"sha256-23"},{"name":"bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"23.11-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11","tag_status":"active","digest":"sha256-23"},{"name":"23-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"}]}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(
        output.suggestions[0].builds,
        [
            "latest".to_owned(),
            "23".to_owned(),
            "23-bookworm".to_owned(),
            "23.11".to_owned(),
            "23.11-bookworm".to_owned(),
            "23.11.0".to_owned(),
            "23.11.0-bookworm".to_owned(),
            "bookworm".to_owned(),
            "current".to_owned(),
            "current-bookworm".to_owned(),
        ]
    );
}

#[test]
fn docker_untagged_image_uses_latest_alias_as_current() {
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
            uri: "file:///Dockerfile".to_owned(),
            language_id: "dockerfile".to_owned(),
            text: package_file_fixture(
                "docker-untagged-image-uses-latest-alias-as-currentDockerfile",
            ),
            workspace_root: None,
        },
        &[node_same_digest_response()],
    );

    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("23.11.0"));
    assert_eq!(output.suggestions[0].builds, node_same_digest_builds());
}

#[test]
fn docker_untagged_image_with_non_version_latest_is_no_match() {
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
            uri: "file:///Dockerfile".to_owned(),
            language_id: "dockerfile".to_owned(),
            text: package_file_fixture(
                "docker-untagged-image-with-non-version-latest-is-no-matchDockerfile",
            ),
            workspace_root: None,
        },
        &[mssql_latest_response()],
    );

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.suggestions[0].builds.is_empty());
}

#[test]
fn docker_explicit_latest_non_version_alias_keeps_latest_as_current() {
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
            uri: "file:///Dockerfile".to_owned(),
            language_id: "dockerfile".to_owned(),
            text: package_file_fixture(
                "docker-explicit-latest-non-version-alias-keeps-latest-as-currentDockerfile",
            ),
            workspace_root: None,
        },
        &[mssql_latest_response()],
    );

    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("latest"));
    assert_eq!(
        output.suggestions[0].builds,
        [
            "latest".to_owned(),
            "2022-RTM-CU2-ubuntu-20.04".to_owned(),
            "2022-RTM-GDR1-ubuntu-20.04".to_owned(),
            "2022-RTM-ubuntu-20.04".to_owned(),
            "2022-latest".to_owned(),
            "2022-preview-ubuntu-22.04".to_owned(),
        ]
    );
}

#[test]
fn docker_same_digest_short_alias_keeps_current_status_and_build_suggestions() {
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
            uri: "file:///Dockerfile".to_owned(),
            language_id: "dockerfile".to_owned(),
            text: package_file_fixture("docker-same-digest-short-alias-keeps-current-status-and-build-suggestionsDockerfile"),
            workspace_root: None,
        },
        &[node_same_digest_response()],
    );

    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("23.11.0"));
    assert_eq!(output.suggestions[0].builds, node_same_digest_builds());
}

fn node_same_digest_response() -> RegistryResponseInput {
    RegistryResponseInput {
        package: "node".to_owned(),
        ecosystem: Docker,
        body: r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"current-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"current","tag_status":"active","digest":"sha256-23"},{"name":"bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"23.11-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11","tag_status":"active","digest":"sha256-23"},{"name":"23-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"}]}"#
            .to_owned(),
    }
}

fn mssql_latest_response() -> RegistryResponseInput {
    RegistryResponseInput {
        package: "mssql/server".to_owned(),
        ecosystem: Docker,
        body: r#"{"results":[{"name":"2022-RTM-CU2-ubuntu-20.04","tag_status":"active","digest":"sha256-a"},{"name":"2022-RTM-GDR1-ubuntu-20.04","tag_status":"active","digest":"sha256-b"},{"name":"2022-RTM-ubuntu-20.04","tag_status":"active","digest":"sha256-c"},{"name":"2022-latest","tag_status":"active","digest":"sha256-latest"},{"name":"2022-preview-ubuntu-22.04","tag_status":"active","digest":"sha256-d"},{"name":"latest","tag_status":"active","digest":"sha256-latest"},{"name":"latest-ubuntu","tag_status":"active","digest":"sha256-e"}]}"#
            .to_owned(),
    }
}

fn node_same_digest_builds() -> Vec<String> {
    [
        "latest",
        "23",
        "23-bookworm",
        "23.11",
        "23.11-bookworm",
        "23.11.0",
        "23.11.0-bookworm",
        "bookworm",
        "current",
        "current-bookworm",
    ]
    .into_iter()
    .map(|value| value.to_owned())
    .collect()
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/core/docker/response/tests")
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
