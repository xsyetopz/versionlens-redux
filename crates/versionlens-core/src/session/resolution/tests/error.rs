use std::fs::read_to_string;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::thread::spawn;

use super::{
    DocumentInput, ProviderSettings, RegistryResponseInput, RegistryUrlConfig,
    session_with_settings, session_without_vulnerabilities,
};
use versionlens_parsers::Ecosystem::{Hex, Npm};

#[test]
fn registry_response_without_latest_creates_no_match() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("registry-response-without-latest-creates-no-match.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"versions":{}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
}

#[test]
fn npm_error_registry_response_creates_error_suggestion() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("npm-error-registry-response-creates-error-suggestion.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"status":"E404"}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "error");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("not found"));
    assert!(output.edits.is_empty());
}

#[test]
fn hex_error_registry_response_creates_error_suggestion() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///mix.exs".to_owned(),
            language_id: "elixir".to_owned(),
            text: package_file_fixture("hex-error-registry-response-creates-error-suggestion.exs"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "plug".to_owned(),
            ecosystem: Hex,
            body: r#"{"status":404}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "error");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("not found"));
    assert!(output.edits.is_empty());
}

#[test]
fn hex_rate_limited_registry_response_creates_error_suggestion() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///mix.exs".to_owned(),
            language_id: "elixir".to_owned(),
            text: package_file_fixture(
                "hex-rate-limited-registry-response-creates-error-suggestion.exs",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "plug".to_owned(),
            ecosystem: Hex,
            body: r#"{"status":429}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "error");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("too many requests")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn registry_responses_try_next_matching_body_when_first_has_no_latest() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "registry-responses-try-next-matching-body-when-first-has-no-latest.json",
            ),
            workspace_root: None,
        },
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Npm,
                body: r#"{"versions":{}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn unauthorized_registry_response_reports_auth_request_urls() {
    let listener = crate::tcp_listener_bind("127.0.0.1:0").expect("bind test server");
    let base_url = format!("http://{}", listener.local_addr().expect("server address"));
    let server = spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer).expect("read request");
        stream
            .write_all(
                b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("write response");
    });
    let session = session_with_settings(
        ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Npm,
                url: base_url.clone(),
            }],
            ..crate::default()
        },
        false,
    );

    let output = session.resolve_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("unauthorized-registry-response-reports-auth-request-urls.json"),
        workspace_root: None,
    });
    server.join().expect("server thread");

    assert_eq!(output.authorization_required_count, 1);
    assert_eq!(output.authorization_required_requests.len(), 1);
    assert_eq!(output.authorization_required_requests[0].auth_url, base_url);
    assert_eq!(
        output.authorization_required_requests[0].request_url,
        format!("{base_url}/left-pad")
    );
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/error")
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
