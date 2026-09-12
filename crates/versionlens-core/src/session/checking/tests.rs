use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use super::{WorkspaceCheckEvent, WorkspaceCheckResult, WorkspaceCheckingOptions};
use crate::{SessionConfigInput, WorkspaceDiscoveryOptions, version_lens_session};

fn options(root: &Path) -> WorkspaceCheckingOptions {
    WorkspaceCheckingOptions::new(WorkspaceDiscoveryOptions::new(vec![root.to_path_buf()]))
}

fn next_document(
    receiver: &Receiver<WorkspaceCheckEvent>,
    generation: u64,
) -> WorkspaceCheckResult {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let event = receiver
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
            .expect("workspace result before deadline");
        if event.generation == generation
            && matches!(event.result, WorkspaceCheckResult::Document { .. })
        {
            return event.result;
        }
    }
}

fn workflow(root: &Path) -> std::path::PathBuf {
    let directory = root.join(".github/workflows");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("ci.yml");
    std::fs::write(&path, "jobs: {check: {steps: [{uses: './missing'}]}}\n").unwrap();
    path
}

#[test]
fn unopened_files_retry_after_expiry_without_adapter_requests() {
    let root = versionlens_test_support::temporary_directory("versionlens-checking-retry").unwrap();
    workflow(&root);
    let mut config = crate::SessionConfig::from_input(SessionConfigInput::default());
    config.show_vulnerabilities = false;
    config.cache_ttl_ms = 5;
    let session = version_lens_session(config);
    let (sender, receiver) = channel();
    let checking = session
        .start_workspace_checking(options(&root), move |event| {
            let _ = sender.send(event);
        })
        .unwrap();
    let mut checked = 0;
    let mut scans = 0;
    while checked < 2 {
        let event = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        match event.result {
            WorkspaceCheckResult::DiscoverySettled => scans += 1,
            WorkspaceCheckResult::Document { result, .. } => {
                let output = result.unwrap();
                assert_eq!(output.suggestions.len(), 1);
                assert_eq!(output.suggestions[0].status, "error");
                checked += 1;
            }
            _ => {}
        }
    }
    assert_eq!(scans, 1);
    checking.stop();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn replacement_uses_the_new_unsaved_snapshot_and_generation() {
    let root =
        versionlens_test_support::temporary_directory("versionlens-checking-overlay").unwrap();
    let path = workflow(&root);
    let session = version_lens_session(
        SessionConfigInput {
            show_vulnerabilities: Some(false),
            ..SessionConfigInput::default()
        }
        .into(),
    );
    let (sender, receiver) = channel();
    let (release, blocked) = channel();
    let checking = session
        .start_workspace_checking(options(&root), move |event| {
            let hold = event.generation == 1
                && matches!(event.result, WorkspaceCheckResult::Document { .. });
            let _ = sender.send(event);
            if hold {
                blocked.recv_timeout(Duration::from_secs(5)).unwrap();
            }
        })
        .unwrap();
    let _ = next_document(&receiver, 1);
    let mut replacement = options(&root);
    replacement.discovery.overlays.push(
        versionlens_model::DocumentInput::new(
            crate::workspace_file_uri(&path).unwrap(),
            "yaml",
            "jobs: {check: {steps: [{uses: './changed'}]}}\n",
            Some(root.to_string_lossy().into_owned()),
        )
        .with_version(2),
    );
    let previous_operation = session.operation_context();
    let generation = checking.replace(replacement).unwrap();
    assert!(!previous_operation.can_publish());
    let current_operation = session.operation_context();
    release.send(()).unwrap();
    assert_eq!(generation, 2);
    let WorkspaceCheckResult::Document { input, result } = next_document(&receiver, generation)
    else {
        unreachable!()
    };
    assert_eq!(input.version, Some(2));
    assert!(input.text.contains("./changed"));
    assert_eq!(result.unwrap().suggestions[0].status, "error");
    assert!(current_operation.can_publish());
    let active = session.operation_context();
    checking.stop();
    assert!(active.can_publish());
    assert!(checking.replace(options(&root)).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn snapshot_budget_produces_a_specific_result_and_settles() {
    let root =
        versionlens_test_support::temporary_directory("versionlens-checking-budget").unwrap();
    workflow(&root);
    let session = version_lens_session(SessionConfigInput::default().into());
    let mut input = options(&root);
    input.max_snapshot_bytes = 0;
    let (sender, receiver) = channel();
    let checking = session
        .start_workspace_checking(input, move |event| {
            let _ = sender.send(event);
        })
        .unwrap();
    let event = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(
        event.result,
        WorkspaceCheckResult::SnapshotLimit { limit: 0, .. }
    ));
    let event = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(
        event.result,
        WorkspaceCheckResult::DiscoverySettled
    ));
    let event = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(event.result, WorkspaceCheckResult::Settled));
    checking.stop();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cache_invalidation_triggers_checking_and_stop_releases_the_listener() {
    let root = versionlens_test_support::temporary_directory("versionlens-checking-clear").unwrap();
    workflow(&root);
    let session = version_lens_session(
        SessionConfigInput {
            show_vulnerabilities: Some(false),
            ..SessionConfigInput::default()
        }
        .into(),
    );
    let (sender, receiver) = channel();
    let operation = session.operation_context();
    let checking = session
        .start_workspace_checking(options(&root), move |event| {
            let _ = sender.send(event);
        })
        .unwrap();
    let _ = next_document(&receiver, 1);
    assert!(operation.can_publish());
    session.clear_cache();
    assert!(!operation.can_publish());
    let WorkspaceCheckResult::Document { result, .. } = next_document(&receiver, 1) else {
        unreachable!()
    };
    assert_eq!(result.unwrap().suggestions[0].status, "error");
    checking.stop();
    while let Ok(event) = receiver.recv_timeout(Duration::from_secs(2)) {
        assert!(!matches!(
            event.result,
            WorkspaceCheckResult::Document { .. }
        ));
    }
    assert!(matches!(
        receiver.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Disconnected)
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn discovery_failures_recover_without_an_adapter_request() {
    for missing_root in [true, false] {
        let parent =
            versionlens_test_support::temporary_directory("versionlens-discovery-retry").unwrap();
        let root = parent.join("workspace");
        if !missing_root {
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(workflow(&root), [0xff]).unwrap();
        }
        let session = version_lens_session(
            SessionConfigInput {
                show_vulnerabilities: Some(false),
                ..SessionConfigInput::default()
            }
            .into(),
        );
        let (sender, receiver) = channel();
        let checking = session
            .start_workspace_checking(options(&root), move |event| {
                let _ = sender.send(event);
            })
            .unwrap();
        let failure = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(
            failure.result,
            WorkspaceCheckResult::DiscoveryFailure(_)
        ));
        workflow(&root);
        let WorkspaceCheckResult::Document { input, result } = next_document(&receiver, 1) else {
            unreachable!()
        };
        assert!(input.uri.ends_with("/ci.yml"));
        assert_eq!(result.unwrap().suggestions[0].status, "error");
        checking.stop();
        std::fs::remove_dir_all(parent).unwrap();
    }
}

#[test]
fn discovery_retries_preserve_failed_overlay_precedence() {
    let root = versionlens_test_support::temporary_directory("versionlens-retry-overlay").unwrap();
    let path = workflow(&root);
    std::fs::write(&path, [0xff]).unwrap();
    let shadow = path.with_file_name("shadow.yml");
    std::fs::write(&shadow, "jobs: {check: {steps: [{uses: './disk'}]}}\n").unwrap();
    let mut request = options(&root);
    request.discovery.limits.max_file_size = 64;
    request
        .discovery
        .overlays
        .push(versionlens_model::DocumentInput::new(
            crate::workspace_file_uri(&shadow).unwrap(),
            "yaml",
            "x".repeat(128),
            Some(crate::workspace_file_uri(&root).unwrap()),
        ));
    let session = version_lens_session(
        SessionConfigInput {
            show_vulnerabilities: Some(false),
            ..SessionConfigInput::default()
        }
        .into(),
    );
    let (sender, receiver) = channel();
    let checking = session
        .start_workspace_checking(request, move |event| {
            let _ = sender.send(event);
        })
        .unwrap();
    loop {
        let event = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        if matches!(event.result, WorkspaceCheckResult::Settled) {
            break;
        }
        assert!(matches!(
            event.result,
            WorkspaceCheckResult::DiscoveryFailure(_) | WorkspaceCheckResult::DiscoverySettled
        ));
    }
    workflow(&root);
    let WorkspaceCheckResult::Document { input, .. } = next_document(&receiver, 1) else {
        unreachable!()
    };
    assert!(input.uri.ends_with("/ci.yml"));
    let settled = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(settled.result, WorkspaceCheckResult::Settled));
    checking.stop();
    std::fs::remove_dir_all(root).unwrap();
}
