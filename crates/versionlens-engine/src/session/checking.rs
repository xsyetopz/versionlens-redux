use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::PoisonError;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use versionlens_model::DocumentInput;

use super::{VersionLensSession, WorkspaceDiscoveryFailure, WorkspaceDiscoveryOptions};
use crate::ResolveDocumentOutput;

mod runner;

/// The input of one run of workspace checking.
#[derive(Debug)]
pub struct WorkspaceCheckingOptions {
    /// How the documents are found.
    pub discovery: WorkspaceDiscoveryOptions,
    /// The most text, in bytes, kept for documents the checking has not read yet.
    pub max_snapshot_bytes: usize,
}

impl WorkspaceCheckingOptions {
    /// Options that check what `discovery` finds.
    pub fn new(discovery: WorkspaceDiscoveryOptions) -> Self {
        Self {
            discovery,
            max_snapshot_bytes: 128 * 1024 * 1024,
        }
    }
}

/// A result of workspace checking, with the generation it belongs to.
#[derive(Debug)]
pub struct WorkspaceCheckEvent {
    /// The generation of the options the result belongs to.
    pub generation: u64,
    /// The result.
    pub result: WorkspaceCheckResult,
}

/// One result of workspace checking.
#[derive(Debug)]
pub enum WorkspaceCheckResult {
    /// A document was checked.
    Document {
        /// The document.
        input: Box<DocumentInput>,
        /// The resolution, or the reason it failed.
        result: Result<ResolveDocumentOutput, String>,
    },
    /// Discovery failed on a path.
    DiscoveryFailure(WorkspaceDiscoveryFailure),
    /// A document does not fit in the snapshot.
    SnapshotLimit {
        /// The URI of the document.
        uri: String,
        /// The snapshot limit, in bytes.
        limit: usize,
    },
    /// A rescanned file that is gone or that a scan now skips.
    Removed {
        /// The URI of the file.
        uri: String,
    },
    /// Discovery found every document.
    DiscoverySettled,
    /// Every found document was checked.
    Settled,
}

#[derive(Debug, Default)]
struct Pending {
    /// A full replacement restarts discovery.
    options: Option<WorkspaceCheckingOptions>,
    /// The complete set of unsaved documents, applied without rediscovery.
    documents: Option<Vec<DocumentInput>>,
    /// Files changed on disk, checked again one by one.
    paths: BTreeSet<PathBuf>,
}

impl Pending {
    fn is_empty(&self) -> bool {
        self.options.is_none() && self.documents.is_none() && self.paths.is_empty()
    }
}

#[derive(Debug)]
struct Shared {
    pending: Mutex<Pending>,
    ready: Condvar,
    generation: AtomicU64,
    closed: AtomicBool,
    max_snapshot_bytes: AtomicUsize,
}

/// A running workspace check. Dropping it does not stop the check; call `stop`.
#[derive(Debug)]
pub struct WorkspaceChecking {
    shared: Arc<Shared>,
    session: VersionLensSession,
}

impl VersionLensSession {
    /// Resolution tasks run on `runtime`, which the caller owns and keeps alive
    /// for as long as checking runs.
    ///
    /// # Errors
    ///
    /// Returns the error of spawning the worker thread.
    pub fn start_workspace_checking(
        &self,
        options: WorkspaceCheckingOptions,
        runtime: tokio::runtime::Handle,
        on_event: impl Fn(WorkspaceCheckEvent) + Send + 'static,
    ) -> std::io::Result<WorkspaceChecking> {
        establish_snapshot(self, &options, true);
        let max_snapshot_bytes = AtomicUsize::new(options.max_snapshot_bytes);
        let shared = Arc::new(Shared {
            pending: Mutex::new(Pending {
                options: Some(options),
                ..Pending::default()
            }),
            ready: Condvar::new(),
            generation: AtomicU64::new(1),
            closed: AtomicBool::new(false),
            max_snapshot_bytes,
        });
        let session = self.clone();
        let worker_shared = Arc::clone(&shared);
        thread::Builder::new()
            .name("versionlens-workspace".to_owned())
            .spawn(move || runner::Runner::new(session, worker_shared, on_event, runtime).run())?;
        Ok(WorkspaceChecking {
            shared,
            session: self.clone(),
        })
    }
}

impl WorkspaceChecking {
    /// The current generation.
    pub fn generation(&self) -> u64 {
        self.shared.generation.load(Ordering::Acquire)
    }

    /// Replaces pending input atomically. Consumers must compare each event's
    /// generation with the current generation before rendering or applying it.
    ///
    /// # Errors
    ///
    /// Returns a message when checking is stopped or the generation limit is reached.
    pub fn replace(&self, options: WorkspaceCheckingOptions) -> Result<u64, &'static str> {
        let mut pending = self
            .shared
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if self.shared.closed.load(Ordering::Acquire) {
            return Err("workspace checking is stopped");
        }
        let previous = self
            .shared
            .generation
            .try_update(Ordering::AcqRel, Ordering::Acquire, |generation| {
                generation.checked_add(1)
            })
            .map_err(|_| "workspace generation limit reached")?;
        establish_snapshot(&self.session, &options, false);
        self.shared
            .max_snapshot_bytes
            .store(options.max_snapshot_bytes, Ordering::Release);
        *pending = Pending {
            options: Some(options),
            ..Pending::default()
        };
        drop(pending);
        self.shared.ready.notify_one();
        Ok(previous + 1)
    }

    /// Replaces the unsaved documents without restarting discovery. Only
    /// documents whose text or version changed are checked again, and the
    /// generation stays the same.
    ///
    /// # Errors
    ///
    /// Returns a message when checking is stopped.
    pub fn update_documents(&self, documents: Vec<DocumentInput>) -> Result<(), &'static str> {
        let mut pending = self
            .shared
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if self.shared.closed.load(Ordering::Acquire) {
            return Err("workspace checking is stopped");
        }
        self.session.set_workspace_documents(snapshot_documents(
            &documents,
            self.shared.max_snapshot_bytes.load(Ordering::Acquire),
        ));
        if let Some(options) = pending.options.as_mut() {
            options.discovery.overlays = documents;
        } else {
            pending.documents = Some(documents);
        }
        drop(pending);
        self.shared.ready.notify_one();
        Ok(())
    }

    /// Checks changed files again without restarting discovery or changing
    /// the generation. A file that is gone or that a scan skips, such as an
    /// ignored one, is reported as removed.
    ///
    /// # Errors
    ///
    /// Returns a message when checking is stopped.
    pub fn rescan_files(&self, paths: Vec<PathBuf>) -> Result<(), &'static str> {
        let mut pending = self
            .shared
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if self.shared.closed.load(Ordering::Acquire) {
            return Err("workspace checking is stopped");
        }
        if pending.options.is_none() {
            pending.paths.extend(paths);
        }
        drop(pending);
        self.shared.ready.notify_one();
        Ok(())
    }

    /// Stops the check.
    pub fn stop(&self) {
        self.shared.closed.store(true, Ordering::Release);
        self.shared.ready.notify_one();
    }
}

fn establish_snapshot(
    session: &VersionLensSession,
    options: &WorkspaceCheckingOptions,
    initial: bool,
) {
    let documents = snapshot_documents(&options.discovery.overlays, options.max_snapshot_bytes);
    if !session.set_workspace_documents(documents) && !initial {
        session.invalidate_workspace();
    }
}

/// The unsaved documents that fit the snapshot budget: all of them, or none.
fn snapshot_documents(
    documents: &[DocumentInput],
    max_snapshot_bytes: usize,
) -> Vec<DocumentInput> {
    let bytes = documents.iter().fold(0_usize, |total, input| {
        total.saturating_add(runner::snapshot_weight(input))
    });
    if bytes <= max_snapshot_bytes {
        documents.to_vec()
    } else {
        Vec::new()
    }
}

impl Drop for WorkspaceChecking {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::LazyLock;
    use std::sync::mpsc::{Receiver, channel};
    use std::time::Duration;

    use super::{WorkspaceCheckEvent, WorkspaceCheckResult, WorkspaceCheckingOptions};
    use crate::{SessionConfig, WorkspaceDiscoveryOptions, version_lens_session};

    fn options(root: &Path) -> WorkspaceCheckingOptions {
        WorkspaceCheckingOptions::new(WorkspaceDiscoveryOptions::new(vec![root.to_path_buf()]))
    }

    fn runtime() -> tokio::runtime::Handle {
        static RUNTIME: LazyLock<tokio::runtime::Runtime> =
            LazyLock::new(|| tokio::runtime::Runtime::new().expect("test runtime"));
        RUNTIME.handle().clone()
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
        let root =
            versionlens_test_support::temporary_directory("versionlens-checking-retry").unwrap();
        workflow(&root);
        let config = SessionConfig {
            show_vulnerabilities: false,
            cache_ttl_ms: 5,
            ..SessionConfig::default()
        };
        let session = version_lens_session(config);
        let (sender, receiver) = channel();
        let checking = session
            .start_workspace_checking(options(&root), runtime(), move |event| {
                drop(sender.send(event));
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
        let session = version_lens_session(SessionConfig {
            show_vulnerabilities: false,
            ..SessionConfig::default()
        });
        let (sender, receiver) = channel();
        let (release, blocked) = channel();
        let checking = session
            .start_workspace_checking(options(&root), runtime(), move |event| {
                let hold = event.generation == 1
                    && matches!(event.result, WorkspaceCheckResult::Document { .. });
                drop(sender.send(event));
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
        // Unsaved edits keep earlier resolution work valid.
        assert!(previous_operation.can_publish());
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
        let session = version_lens_session(SessionConfig::default());
        let mut input = options(&root);
        input.max_snapshot_bytes = 0;
        let (sender, receiver) = channel();
        let checking = session
            .start_workspace_checking(input, runtime(), move |event| {
                drop(sender.send(event));
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
        let root =
            versionlens_test_support::temporary_directory("versionlens-checking-clear").unwrap();
        workflow(&root);
        let session = version_lens_session(SessionConfig {
            show_vulnerabilities: false,
            ..SessionConfig::default()
        });
        let (sender, receiver) = channel();
        let operation = session.operation_context();
        let checking = session
            .start_workspace_checking(options(&root), runtime(), move |event| {
                drop(sender.send(event));
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
                versionlens_test_support::temporary_directory("versionlens-discovery-retry")
                    .unwrap();
            let root = parent.join("workspace");
            if !missing_root {
                std::fs::create_dir_all(&root).unwrap();
                std::fs::write(workflow(&root), [0xff]).unwrap();
            }
            let session = version_lens_session(SessionConfig {
                show_vulnerabilities: false,
                ..SessionConfig::default()
            });
            let (sender, receiver) = channel();
            let checking = session
                .start_workspace_checking(options(&root), runtime(), move |event| {
                    drop(sender.send(event));
                })
                .unwrap();
            let failure = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(matches!(
                failure.result,
                WorkspaceCheckResult::DiscoveryFailure(_)
            ));
            workflow(&root);
            let WorkspaceCheckResult::Document { input, result } = next_document(&receiver, 1)
            else {
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
        let root =
            versionlens_test_support::temporary_directory("versionlens-retry-overlay").unwrap();
        let path = workflow(&root);
        std::fs::write(&path, [0xff]).unwrap();
        let shadow = path.with_file_name("shadow.yml");
        std::fs::write(&shadow, "jobs: {check: {steps: [{uses: './disk'}]}}\n").unwrap();
        let mut request = options(&root);
        request.discovery.limits.file_size = 64;
        request
            .discovery
            .overlays
            .push(versionlens_model::DocumentInput::new(
                crate::workspace_file_uri(&shadow).unwrap(),
                "yaml",
                "x".repeat(128),
                Some(crate::workspace_file_uri(&root).unwrap()),
            ));
        let session = version_lens_session(SessionConfig {
            show_vulnerabilities: false,
            ..SessionConfig::default()
        });
        let (sender, receiver) = channel();
        let checking = session
            .start_workspace_checking(request, runtime(), move |event| {
                drop(sender.send(event));
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

    #[test]
    fn document_updates_recheck_only_the_edited_document_without_rediscovery() {
        let root =
            versionlens_test_support::temporary_directory("versionlens-checking-update").unwrap();
        let path = workflow(&root);
        let other = root.join(".github/workflows/other.yml");
        std::fs::write(&other, "jobs: {check: {steps: [{uses: './other'}]}}\n").unwrap();
        let session = version_lens_session(SessionConfig {
            show_vulnerabilities: false,
            ..SessionConfig::default()
        });
        let (sender, receiver) = channel();
        let checking = session
            .start_workspace_checking(options(&root), runtime(), move |event| {
                drop(sender.send(event));
            })
            .unwrap();
        let mut checked = 0;
        let mut scans = 0;
        while checked < 2 || scans < 1 {
            let event = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
            match event.result {
                WorkspaceCheckResult::DiscoverySettled => scans += 1,
                WorkspaceCheckResult::Document { .. } => checked += 1,
                _ => {}
            }
        }
        let generation = checking.generation();

        let edited = versionlens_model::DocumentInput::new(
            crate::workspace_file_uri(&path).unwrap(),
            "yaml",
            "jobs: {check: {steps: [{uses: './changed'}]}}\n",
            Some(root.to_string_lossy().into_owned()),
        )
        .with_version(2);
        checking.update_documents(vec![edited]).unwrap();
        assert_eq!(checking.generation(), generation);

        let deadline = std::time::Instant::now() + Duration::from_millis(1500);
        let mut rechecked = Vec::new();
        while let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) {
            let Ok(event) = receiver.recv_timeout(remaining) else {
                break;
            };
            match event.result {
                WorkspaceCheckResult::DiscoverySettled => scans += 1,
                WorkspaceCheckResult::Document { input, .. } => rechecked.push(input.text),
                _ => {}
            }
        }
        assert_eq!(scans, 1, "an unsaved edit restarted discovery");
        assert_eq!(
            rechecked.len(),
            1,
            "expected only the edited document: {rechecked:?}"
        );
        assert!(rechecked[0].contains("./changed"));
        checking.stop();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn retry_delays_double_from_one_second_up_to_five_minutes() {
        let delays = (0..12)
            .map(|failures| super::runner::retry_delay(failures).as_secs())
            .collect::<Vec<_>>();
        assert_eq!(delays, [1, 2, 4, 8, 16, 32, 64, 128, 256, 300, 300, 300]);
        assert_eq!(super::runner::retry_delay(u32::MAX).as_secs(), 300);
    }

    #[test]
    fn rescanned_files_are_checked_or_removed_without_rediscovery() {
        let root =
            versionlens_test_support::temporary_directory("versionlens-checking-rescan").unwrap();
        let path = workflow(&root);
        let session = version_lens_session(SessionConfig {
            show_vulnerabilities: false,
            ..SessionConfig::default()
        });
        let (sender, receiver) = channel();
        let checking = session
            .start_workspace_checking(options(&root), runtime(), move |event| {
                drop(sender.send(event));
            })
            .unwrap();
        let generation = checking.generation();
        next_document(&receiver, generation);

        std::fs::write(&path, "jobs: {check: {steps: [{uses: './other'}]}}\n").unwrap();
        checking.rescan_files(vec![path.clone()]).unwrap();
        let WorkspaceCheckResult::Document { input, .. } = next_document(&receiver, generation)
        else {
            unreachable!();
        };
        assert!(input.text.contains("./other"));

        std::fs::remove_file(&path).unwrap();
        checking.rescan_files(vec![path]).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let event = receiver
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .expect("removal before deadline");
            assert_eq!(event.generation, generation);
            assert!(!matches!(
                event.result,
                WorkspaceCheckResult::DiscoverySettled
            ));
            if let WorkspaceCheckResult::Removed { uri } = event.result {
                assert_eq!(uri, input.uri);
                break;
            }
        }
        checking.stop();
        std::fs::remove_dir_all(root).unwrap();
    }
}
