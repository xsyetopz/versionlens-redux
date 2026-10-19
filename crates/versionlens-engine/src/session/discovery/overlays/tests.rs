use versionlens_model::DocumentInput;

use crate::{
    SessionConfig, WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryFileSize,
    WorkspaceDiscoveryOptions, enabled_provider_config_from_name, version_lens_session,
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
    let session = version_lens_session(SessionConfig::default());
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
    let session = version_lens_session(SessionConfig {
        enabled_providers: vec![
            enabled_provider_config_from_name("cargo").expect("cargo provider"),
        ],
        ..SessionConfig::default()
    });
    assert!(
        session
            .discover_workspace_documents(options())
            .next()
            .is_none()
    );
    let session = version_lens_session(SessionConfig::default());
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

#[test]
fn open_documents_outside_the_workspace_roots_are_still_checked() {
    let root =
        versionlens_test_support::temporary_directory("versionlens-outside-overlay").unwrap();
    let session = version_lens_session(SessionConfig::default());
    let mut input = options();
    input.roots = vec![root.clone()];
    let expected = input.overlays[0].clone();
    let documents = session
        .discover_workspace_documents(input)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(documents, [expected]);
    std::fs::remove_dir_all(root).unwrap();
}
