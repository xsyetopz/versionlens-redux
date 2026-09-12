use versionlens_model::DocumentInput;

use crate::{
    SessionConfigInput, WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryFileSize,
    WorkspaceDiscoveryOptions, version_lens_session,
};

fn options() -> WorkspaceDiscoveryOptions {
    let mut options = WorkspaceDiscoveryOptions::new(vec![]);
    options.overlays.push(
        DocumentInput::new(
            "file:///standalone/package.json",
            "json",
            r#"{"dependencies":{"example":"1.0.0"}}"#,
            None,
        )
        .with_version(7),
    );
    options
}

#[test]
fn standalone_discovery_preserves_the_supplied_snapshot_without_filesystem_roots() {
    let session = version_lens_session(SessionConfigInput::default().into());
    let input = options();
    let expected = input.overlays[0].clone();
    let documents = session
        .discover_workspace_documents(input)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(documents, [expected]);
}

#[test]
fn standalone_discovery_applies_provider_enablement_and_size_limits() {
    let session = version_lens_session(
        SessionConfigInput {
            enabled_providers: Some(vec!["cargo".to_owned()]),
            ..SessionConfigInput::default()
        }
        .into(),
    );
    assert!(
        session
            .discover_workspace_documents(options())
            .next()
            .is_none()
    );
    let session = version_lens_session(SessionConfigInput::default().into());
    let mut excluded = options();
    excluded.exclusions.push("**/package.json".to_owned());
    assert!(
        session
            .discover_workspace_documents(excluded)
            .next()
            .is_some()
    );
    let mut oversized = options();
    oversized.limits.max_file_size = 1;
    let expected_size = u64::try_from(oversized.overlays[0].text.len()).unwrap();
    let failure = session
        .discover_workspace_documents(oversized)
        .next()
        .unwrap()
        .unwrap_err();
    assert_eq!(
        failure.kind,
        WorkspaceDiscoveryFailureKind::FileTooLarge(Box::new(WorkspaceDiscoveryFileSize {
            size: expected_size,
            limit: 1,
        }))
    );
}
