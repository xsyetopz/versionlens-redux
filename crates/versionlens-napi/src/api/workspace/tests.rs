use std::time::{Duration, Instant};

use super::WorkspaceClient;
use crate::binding::{NativeDocumentInput, NativeWorkspaceCheckingInput};

fn input(version: u32) -> NativeWorkspaceCheckingInput {
    NativeWorkspaceCheckingInput {
        roots: vec![],
        exclusions: vec![],
        provider_exclusions: None,
        documents: vec![NativeDocumentInput {
            uri: "file:///work/.github/workflows/ci.yml".to_owned(),
            language_id: "yaml".to_owned(),
            text: format!("jobs: {{check: {{steps: [{{uses: './action-{version}'}}]}}}}\n"),
            workspace_root: None,
            version: Some(version),
        }],
    }
}

#[test]
fn native_workspace_snapshots_deduplicate_and_reject_queued_older_generations() {
    let session = versionlens_core::version_lens_session(
        versionlens_core::SessionConfigInput {
            show_vulnerabilities: Some(false),
            ..versionlens_core::SessionConfigInput::default()
        }
        .into(),
    );
    let mut client = WorkspaceClient::default();
    assert_eq!(
        client.check(&session, input(1)).unwrap(),
        ("1".to_owned(), true)
    );
    assert_eq!(
        client.check(&session, input(1)).unwrap(),
        ("1".to_owned(), false)
    );
    assert_eq!(
        client.check(&session, input(2)).unwrap(),
        ("2".to_owned(), true)
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let events = client.take();
        assert!(events.iter().all(|event| event.generation == "2"));
        if let Some(document) = events.into_iter().find_map(|event| event.document) {
            assert_eq!(document.version, Some(2));
            assert!(document.text.contains("action-2"));
            break;
        }
        assert!(Instant::now() < deadline, "workspace completion deadline");
        std::thread::sleep(Duration::from_millis(10));
    }
    client.invalidate().unwrap();
    assert_eq!(client.generation().as_deref(), Some("3"));
    client.close();
    assert!(client.generation().is_none());
    assert!(client.take().is_empty());
}

#[test]
fn invalid_workspace_roots_fail_before_a_controller_starts() {
    let session = versionlens_core::version_lens_session(
        versionlens_core::SessionConfigInput::default().into(),
    );
    let mut client = WorkspaceClient::default();
    let mut options = input(1);
    options
        .roots
        .push("https://example.test/workspace".to_owned());
    assert!(
        client
            .check(&session, options)
            .unwrap_err()
            .contains("invalid workspace root")
    );
    assert!(client.generation().is_none());
}
