use crate::binding::{NativeDocumentInput, NativeSessionConfig};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::create_session;

fn session_config() -> NativeSessionConfig {
    NativeSessionConfig {
        cache_duration_minutes: None,
        enabled_providers: None,
        providers: None,
        suggestion_indicators: None,
        show_vulnerabilities: None,
        show_suggestion_stats: None,
        show_prereleases: false,
        http: None,
    }
}

fn package_document() -> NativeDocumentInput {
    NativeDocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-document.json").to_owned(),
        workspace_root: None,
    }
}

#[test]
fn dispose_session_releases_inner_session() {
    let session = create_session(session_config());
    let output = session.analyze_document(package_document());
    assert!(output.is_supported_manifest);
    assert_eq!(output.active_provider_name, Some("npm".to_owned()));

    session.dispose_session();
    session.clear_cache();

    let output = session.analyze_document(package_document());
    assert!(!output.is_supported_manifest);
    assert!(!output.status.visible);
}

#[test]
fn dispose_session_does_not_wait_for_an_in_flight_session_owner() {
    let session = create_session(session_config());
    let in_flight = session.session().expect("session should be available");
    let (sender, receiver) = mpsc::channel();

    thread::scope(|scope| {
        scope.spawn(|| {
            session.dispose_session();
            sender.send(()).expect("receiver should remain available");
        });

        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("dispose should only detach the session cell");
        assert!(session.session().is_none());
        assert!(
            in_flight
                .analyze_document(package_document().into_core())
                .is_supported_manifest
        );
    });
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-napi/src/api/tests")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
