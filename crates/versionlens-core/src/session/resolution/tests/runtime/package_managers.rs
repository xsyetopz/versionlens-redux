use super::*;

async fn resolve_package_manager(
    name: &str,
    requirement: &str,
    body: impl Into<String>,
) -> ResolveDocumentOutput {
    session_without_vulnerabilities()
        .resolve_document_with_responses(
            DocumentInput::new(
                "file:///work/package.json",
                "json",
                format!(r#"{{"packageManager":"{name}@{requirement}"}}"#),
                None,
            ),
            &[RegistryResponseInput::new(name, Npm, body.into())],
        )
        .await
}

fn sha224_registry_metadata(current_url: &str, selected_url: &str) -> String {
    format!(
        r#"{{"dist-tags":{{"latest":"2.0.0"}},"versions":{{"1.0.0":{{"dist":{{"tarball":"{current_url}"}}}},"2.0.0":{{"dist":{{"tarball":"{selected_url}"}}}}}}}}"#
    )
}

const CURRENT_ARTIFACT_SHA224: &str = "e8c69735a0b5c2320a34aa2234434719d06d2b71a18d74904ebe874c";

#[tokio::test]
async fn package_manager_pins_use_published_package_versions() {
    let output = resolve_package_manager(
        "pnpm",
        "9.1.2",
        r#"{"dist-tags":{"latest":"10.34.4"},"versions":{"9.1.2":{},"10.34.4":{}}}"#,
    )
    .await;
    assert!(!output.edits.is_empty(), "{output:?}");
    assert_eq!(output.edits[0].new_text, "10.34.4");
}

#[tokio::test]
async fn yarn_pins_use_the_release_catalog_for_their_version_family() {
    for (requirement, body, expected) in [
        (
            "1.22.19",
            r#"{"dist-tags":{"latest":"1.22.22"},"versions":{"1.22.19":{},"1.22.22":{}}}"#,
            "1.22.22",
        ),
        (
            "4.1.0",
            r#"{"aliases":{"latest":"4.2.0","stable":"4.1.1"},"tags":["4.2.0","4.1.1","4.1.0"]}"#,
            "4.2.0",
        ),
    ] {
        let output = resolve_package_manager("yarn", requirement, body).await;
        assert_eq!(output.edits.len(), 1, "{output:?}");
        assert_eq!(output.edits[0].new_text, expected);
    }
}

#[tokio::test]
async fn yarn_aliases_do_not_replace_a_newer_pinned_release() {
    let output = resolve_package_manager(
        "yarn",
        "4.3.0",
        r#"{"aliases":{"latest":"4.2.0"},"tags":["4.3.0","4.2.0"]}"#,
    )
    .await;
    assert!(output.edits.is_empty(), "{output:?}");
    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("4.3.0"));
}

#[tokio::test]
async fn yarn_classic_sha1_pins_are_verified_and_preserved() {
    let current = "00112233445566778899aabbccddeeff00112233";
    let selected = "ffeeddccbbaa99887766554433221100ffeeddcc";
    let body = format!(
        r#"{{"dist-tags":{{"latest":"1.22.22"}},"versions":{{"1.22.19":{{"dist":{{"shasum":"{current}"}}}},"1.22.22":{{"dist":{{"shasum":"{selected}"}}}}}}}}"#
    );
    let output = resolve_package_manager("yarn", &format!("1.22.19+sha1.{current}"), body).await;
    assert_eq!(output.edits.len(), 1, "{output:?}");
    assert_eq!(output.edits[0].new_text, format!("1.22.22+sha1.{selected}"));
}

#[tokio::test]
async fn sha224_pins_hash_the_current_and_selected_registry_artifacts() {
    let (base_url, server) =
        super::github::github_api_server(vec![(200, "current"), (200, "selected")]);
    let current_url = format!("{base_url}current.tgz");
    let selected_url = format!("{base_url}selected.tgz");
    let body = sha224_registry_metadata(&current_url, &selected_url);
    let output = resolve_package_manager(
        "pnpm",
        &format!("1.0.0+sha224.{CURRENT_ARTIFACT_SHA224}"),
        body,
    )
    .await;
    assert_eq!(output.edits.len(), 1, "{output:?}");
    assert_eq!(
        output.edits[0].new_text,
        "2.0.0+sha224.1bb4223297e1f5140bf49830b53cd9e844a5142a3dfa5f8a2b024aa9"
    );
    assert_eq!(
        server.join().unwrap(),
        ["/repos/current.tgz", "/repos/selected.tgz"]
    );
}

#[tokio::test]
async fn sha224_rejects_a_bad_current_pin_before_fetching_the_target() {
    let (base_url, server) = super::github::github_api_server(vec![(200, "current")]);
    let body = sha224_registry_metadata(
        &format!("{base_url}current.tgz"),
        &format!("{base_url}selected.tgz"),
    );
    let output =
        resolve_package_manager("pnpm", &format!("1.0.0+sha224.{}", "00".repeat(28)), body).await;
    assert!(output.edits.is_empty(), "{output:?}");
    assert_eq!(output.suggestions[0].status, "error");
    assert!(
        output.suggestions[0]
            .latest
            .as_deref()
            .unwrap()
            .contains("integrity pin does not match")
    );
    assert_eq!(server.join().unwrap(), ["/repos/current.tgz"]);
}

#[tokio::test]
async fn sha224_artifact_fetch_failures_are_explicit() {
    for status in [401, 404] {
        let (base_url, server) = super::github::github_api_server(vec![(status, "missing")]);
        let current_url = format!("{base_url}current.tgz");
        let body = sha224_registry_metadata(&current_url, &format!("{base_url}selected.tgz"));
        let output = resolve_package_manager(
            "pnpm",
            &format!("1.0.0+sha224.{CURRENT_ARTIFACT_SHA224}"),
            body,
        )
        .await;
        assert!(output.edits.is_empty(), "{output:?}");
        assert_eq!(output.suggestions[0].status, "error");
        assert!(output.suggestions[0].latest.is_some());
        assert_eq!(server.join().unwrap(), ["/repos/current.tgz"]);
        if status == 401 {
            assert_eq!(output.authorization_required_count, 1);
            assert_eq!(
                output.authorization_required_requests[0].request_url,
                current_url
            );
        } else {
            assert_eq!(output.authorization_required_count, 0);
        }
    }
}

#[tokio::test]
async fn package_manager_integrity_pins_are_verified_and_preserved() {
    let current_digest = "00".repeat(64);
    let target_digest = "ff".repeat(64);
    let body = serde_json::json!({
        "dist-tags": {"latest": "10.34.4"},
        "versions": {
            "9.1.2": {"dist": {"integrity": format!("sha512-{}==", "A".repeat(86))}},
            "10.34.4": {"dist": {"integrity": format!("sha512-{}w==", "/".repeat(85))}}
        }
    })
    .to_string();
    let session = session_without_vulnerabilities();
    let input = DocumentInput::new(
        "file:///work/package.json",
        "json",
        format!(r#"{{"packageManager":"pnpm@9.1.2+sha512.{current_digest}"}}"#),
        None,
    );
    let responses = [RegistryResponseInput::new("pnpm", Npm, &body)];
    let output = session
        .resolve_document_with_responses(input, &responses)
        .await;
    assert_eq!(output.edits.len(), 1, "{output:?}");
    assert_eq!(
        output.edits[0].new_text,
        format!("10.34.4+sha512.{target_digest}")
    );
    let current_input = DocumentInput::new(
        "file:///work/package.json",
        "json",
        format!(r#"{{"packageManager":"pnpm@10.34.4+sha512.{target_digest}"}}"#),
        None,
    );
    let current = session
        .resolve_document_with_responses(current_input.clone(), &responses)
        .await;
    assert!(current.edits.is_empty());
    assert_eq!(current.suggestions[0].status, "current");
    assert!(
        session
            .analyze_document(current_input)
            .code_lenses
            .iter()
            .all(|lens| lens.command.is_empty())
    );
    for pin in [format!("sha512.{target_digest}"), "sha224.1234".to_owned()] {
        let input = DocumentInput::new(
            "file:///work/package.json",
            "json",
            format!(r#"{{"packageManager":"pnpm@9.1.2+{pin}"}}"#),
            None,
        );
        let output = session
            .resolve_document_with_responses(input, &responses)
            .await;
        assert!(output.edits.is_empty());
        assert_eq!(output.suggestions[0].status, "error");
    }
}

#[tokio::test]
async fn package_manager_release_channels_follow_published_metadata() {
    let body = r#"{"dist-tags":{"latest":"10.0.0","next":"11.0.0-beta.1"},"versions":{"9.0.0":{},"10.0.0":{},"11.0.0":{},"11.0.0-beta.1":{}}}"#;
    for (requirement, latest, status) in [
        ("9.0.0", "10.0.0", "updateAvailable"),
        ("11.0.0", "11.0.0", "current"),
        ("latest", "10.0.0", "current"),
        ("next", "11.0.0-beta.1", "current"),
    ] {
        let output = session_without_vulnerabilities()
            .resolve_document_with_responses(
                DocumentInput::new(
                    "file:///work/package.json",
                    "json",
                    format!(r#"{{"packageManager":"pnpm@{requirement}"}}"#),
                    None,
                ),
                &[RegistryResponseInput::new("pnpm", Npm, body)],
            )
            .await;
        assert_eq!(
            output.suggestions[0].latest.as_deref(),
            Some(latest),
            "{output:?}"
        );
        assert_eq!(output.suggestions[0].status, status);
        if status == "current" {
            assert!(output.edits.is_empty());
        }
    }
}

#[tokio::test]
async fn package_manager_channels_require_a_published_target() {
    for body in [
        r#"{"versions":{"10.0.0":{}}}"#,
        r#"{"dist-tags":{"latest":"11.0.0"},"versions":{"10.0.0":{}}}"#,
        r#"{"dist-tags":{"latest":"invalid"},"versions":{"invalid":{}}}"#,
    ] {
        let input = DocumentInput::new(
            "file:///work/package.json",
            "json",
            r#"{"packageManager":"pnpm@latest"}"#,
            None,
        );
        let session = session_without_vulnerabilities();
        let output = session
            .resolve_document_with_responses(
                input.clone(),
                &[RegistryResponseInput::new("pnpm", Npm, body)],
            )
            .await;
        assert_eq!(output.suggestions[0].status, "error");
        assert!(output.edits.is_empty());
        assert!(session.document_is_fresh(&input));
    }
}
