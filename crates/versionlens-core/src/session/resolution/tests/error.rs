use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use super::{
    DocumentInput, Ecosystem, ProviderSettings, RegistryResponseInput, RegistryUrlConfig,
    session_with_settings, session_without_vulnerabilities,
};

#[test]
fn registry_response_without_latest_creates_no_match() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
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
            text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"status":"E404"}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "error");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("not found"));
    assert!(output.edits.is_empty());
}

#[test]
fn registry_responses_try_next_matching_body_when_first_has_no_latest() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        &[
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Ecosystem::Npm,
                body: r#"{"versions":{}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Ecosystem::Npm,
                body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn unauthorized_registry_response_reports_auth_request_urls() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let base_url = format!("http://{}", listener.local_addr().expect("server address"));
    let server = thread::spawn(move || {
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
                ecosystem: Ecosystem::Npm,
                url: base_url.clone(),
            }],
            ..ProviderSettings::default()
        },
        false,
    );

    let output = session.resolve_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
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
