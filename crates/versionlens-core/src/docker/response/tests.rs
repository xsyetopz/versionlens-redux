use crate::RegistryResponseInput;
use versionlens_model::DocumentInput;

use versionlens_model::Ecosystem::Docker;

fn assert_node_same_digest(output: &crate::contract::ResolveDocumentOutput) {
    crate::support::tests::assert_suggestion(output, 0, "current", Some("23.11.0"));
    assert_eq!(output.suggestions[0].builds, node_same_digest_builds());
}

#[test]
fn docker_registry_response_missing_requested_tag_creates_no_match() {
    let session = crate::support::tests::test_session(true);
    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///Dockerfile".to_owned(),
            "dockerfile".to_owned(),
            package_file_fixture(
                "docker-registry-response-missing-requested-tag-creates-no-matchDockerfile",
            ),
            None,
        ),
        &[RegistryResponseInput::new(
            "node".to_owned(),
            Docker,
            r#"{"results":[{"name":"2.0.0","tag_status":"active","digest":"sha256-2"}]}"#
                .to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
}

#[test]
fn docker_same_digest_aliases_keep_current_status_and_create_build_suggestions() {
    let session = crate::support::tests::test_session(true);
    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///Dockerfile".to_owned(), "dockerfile".to_owned(), package_file_fixture("docker-same-digest-aliases-keep-current-status-and-create-build-suggestionsDockerfile"), None),
        &[RegistryResponseInput::new("node".to_owned(), Docker, r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"current-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"current","tag_status":"active","digest":"sha256-23"},{"name":"bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"23.11-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11","tag_status":"active","digest":"sha256-23"},{"name":"23-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"}]}"#
                .to_owned())],
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
    let session = crate::support::tests::test_session(true);
    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///Dockerfile".to_owned(),
            "dockerfile".to_owned(),
            package_file_fixture("docker-untagged-image-uses-latest-alias-as-currentDockerfile"),
            None,
        ),
        &[node_same_digest_response()],
    );

    assert_node_same_digest(&output);
}

#[test]
fn docker_untagged_image_with_non_version_latest_is_no_match() {
    let session = crate::support::tests::test_session(true);
    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///Dockerfile".to_owned(),
            "dockerfile".to_owned(),
            package_file_fixture(
                "docker-untagged-image-with-non-version-latest-is-no-matchDockerfile",
            ),
            None,
        ),
        &[mssql_latest_response()],
    );

    crate::support::tests::assert_suggestion(&output, 0, "noMatch", None);
    assert!(output.suggestions[0].builds.is_empty());
}

#[test]
fn docker_explicit_latest_non_version_alias_keeps_latest_as_current() {
    let session = crate::support::tests::test_session(true);
    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///Dockerfile".to_owned(),
            "dockerfile".to_owned(),
            package_file_fixture(
                "docker-explicit-latest-non-version-alias-keeps-latest-as-currentDockerfile",
            ),
            None,
        ),
        &[mssql_latest_response()],
    );

    crate::support::tests::assert_suggestion(&output, 0, "current", Some("latest"));
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
    let session = crate::support::tests::test_session(true);
    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///Dockerfile".to_owned(), "dockerfile".to_owned(), package_file_fixture("docker-same-digest-short-alias-keeps-current-status-and-build-suggestionsDockerfile"), None),
        &[node_same_digest_response()],
    );

    assert_node_same_digest(&output);
}

fn node_same_digest_response() -> RegistryResponseInput {
    RegistryResponseInput::new(
        "node".to_owned(),
        Docker,
        crate::support::tests::fixture("tests/fixtures/shared/docker", "node-same-digest.json"),
    )
}

fn mssql_latest_response() -> RegistryResponseInput {
    RegistryResponseInput::new("mssql/server".to_owned(), Docker, r#"{"results":[{"name":"2022-RTM-CU2-ubuntu-20.04","tag_status":"active","digest":"sha256-a"},{"name":"2022-RTM-GDR1-ubuntu-20.04","tag_status":"active","digest":"sha256-b"},{"name":"2022-RTM-ubuntu-20.04","tag_status":"active","digest":"sha256-c"},{"name":"2022-latest","tag_status":"active","digest":"sha256-latest"},{"name":"2022-preview-ubuntu-22.04","tag_status":"active","digest":"sha256-d"},{"name":"latest","tag_status":"active","digest":"sha256-latest"},{"name":"latest-ubuntu","tag_status":"active","digest":"sha256-e"}]}"#
            .to_owned())
}

fn node_same_digest_builds() -> Vec<String> {
    serde_json::from_str(&crate::support::tests::fixture(
        "tests/fixtures/shared/docker",
        "node-same-digest-builds.json",
    ))
    .expect("shared Docker build fixture must be valid")
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/core-scenarios/docker/response/tests", name)
}
