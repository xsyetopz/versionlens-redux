use super::*;
use crate::workspace::tests::support::TestWorkspace;

fn new_snapshot() -> RegistryFileSnapshot {
    RegistryFileSnapshot::new(std::sync::Arc::new(crate::default()))
}

fn assert_reader_failure_is_scoped(
    snapshot: &RegistryFileSnapshot,
    input: &DocumentInput,
    broken: &Path,
    healthy: &Path,
) {
    let broken_reader = snapshot.reader(input);
    assert_eq!(broken_reader.read(broken), None);
    assert!(broken_reader.failure().is_some());

    let healthy_reader = snapshot.reader(input);
    assert_eq!(healthy_reader.read(healthy).as_deref(), Some("ok"));
    assert!(healthy_reader.failure().is_none());
}

#[test]
fn snapshot_bounds_accumulated_hits_and_misses_without_eviction() {
    let workspace = TestWorkspace::new("registry-snapshot-count");
    let cached = workspace.write("cached", "before");
    let snapshot = new_snapshot();
    assert_eq!(
        snapshot.disk.read(&cached).unwrap().as_deref(),
        Some("before")
    );

    for index in 1..MAX_REGISTRY_SNAPSHOT_ENTRIES {
        assert_eq!(
            snapshot
                .disk
                .read(&workspace.root.join(format!("missing-{index}")))
                .unwrap(),
            None
        );
    }
    assert_eq!(
        snapshot.disk.read(&cached).unwrap().as_deref(),
        Some("before")
    );

    let rejected = workspace.root.join("rejected");
    let failure = snapshot.disk.read(&rejected).unwrap_err();
    assert!(failure.message().contains("no more files can be read"));
    let state = snapshot
        .disk
        .state
        .lock()
        .unwrap_or_else(crate::recover_poison);
    assert_eq!(state.files.len(), MAX_REGISTRY_SNAPSHOT_ENTRIES);
    assert_eq!(
        state.bytes,
        state
            .files
            .iter()
            .map(|(path, value)| registry_entry_bytes(path, value))
            .sum::<usize>()
    );
    drop(state);

    std::fs::write(&cached, "after").unwrap();
    assert_eq!(
        snapshot.disk.read(&cached).unwrap().as_deref(),
        Some("before")
    );
    let next_generation = new_snapshot();
    assert_eq!(
        next_generation.disk.read(&cached).unwrap().as_deref(),
        Some("after")
    );
}

#[test]
fn snapshot_bounds_accumulated_content_bytes() {
    let workspace = TestWorkspace::new("registry-snapshot-bytes");
    let contents = "x".repeat(MAX_REGISTRY_FILE_BYTES);
    let snapshot = new_snapshot();
    let mut accepted = 0;
    let failure = loop {
        let index = accepted;
        let path = workspace.write(&format!("config-{index}"), &contents);
        match snapshot.disk.read(&path) {
            Ok(value) => {
                assert_eq!(value.as_deref(), Some(contents.as_str()));
                accepted += 1;
            }
            Err(failure) => break failure,
        }
    };
    assert!(
        failure
            .message()
            .contains("contents and path metadata exceed")
    );
    let state = snapshot
        .disk
        .state
        .lock()
        .unwrap_or_else(crate::recover_poison);
    assert_eq!(state.files.len(), accepted);
    assert!(state.bytes <= MAX_REGISTRY_SNAPSHOT_BYTES);
    assert_eq!(
        state.bytes,
        state
            .files
            .iter()
            .map(|(path, value)| registry_entry_bytes(path, value))
            .sum::<usize>()
    );
}

#[test]
fn shared_snapshot_rejects_a_sparse_oversized_file() {
    let workspace = TestWorkspace::new("registry-sparse-file");
    let path = workspace.root.join("sparse");
    let file = File::create(&path).unwrap();
    file.set_len(MAX_REGISTRY_FILE_BYTES as u64 + 1).unwrap();

    let snapshot = RegistryFileSnapshot::default();
    let failure = snapshot.disk.read(&path).unwrap_err();
    assert!(failure.message().contains("the limit is"));
}

#[test]
fn disk_failure_is_reported_only_by_readers_that_touch_the_path() {
    let workspace = TestWorkspace::new("registry-reader-disk-failure");
    let broken = workspace.root.join("broken");
    let file = File::create(&broken).unwrap();
    file.set_len(MAX_REGISTRY_FILE_BYTES as u64 + 1).unwrap();
    let healthy = workspace.write("healthy", "ok");
    let input = workspace.document("package.json", "{}", 1);
    let snapshot = new_snapshot();

    assert_reader_failure_is_scoped(&snapshot, &input, &broken, &healthy);
}

#[test]
fn overlay_failure_is_reported_only_by_readers_that_touch_the_path() {
    let workspace = TestWorkspace::new("registry-reader-overlay-failure");
    let broken = workspace.root.join("broken");
    let healthy = workspace.root.join("healthy");
    let documents = std::sync::Arc::new(WorkspaceDocuments::from([
        (
            broken.clone(),
            ("x".repeat(MAX_REGISTRY_FILE_BYTES + 1), Some(1)),
        ),
        (healthy.clone(), ("ok".to_owned(), Some(1))),
    ]));
    let input = workspace.document("package.json", "{}", 1);
    let snapshot = RegistryFileSnapshot::new(documents);

    assert_reader_failure_is_scoped(&snapshot, &input, &broken, &healthy);
}

#[test]
fn required_read_reports_a_missing_path_cached_by_an_optional_read() {
    let workspace = TestWorkspace::new("registry-reader-required-miss");
    let missing = workspace.root.join("missing");
    let input = workspace.document("package.json", "{}", 1);
    let snapshot = new_snapshot();

    let optional_reader = snapshot.reader(&input);
    assert_eq!(optional_reader.read(&missing), None);
    assert!(optional_reader.failure().is_none());

    let required_reader = snapshot.reader(&input);
    assert_eq!(required_reader.read_required(&missing), None);
    assert!(
        required_reader
            .failure()
            .is_some_and(|failure| failure.message().contains("required file was not found"))
    );
}
