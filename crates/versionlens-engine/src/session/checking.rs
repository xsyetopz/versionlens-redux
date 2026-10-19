use std::sync::PoisonError;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use versionlens_model::DocumentInput;

use super::{VersionLensSession, WorkspaceDiscoveryFailure, WorkspaceDiscoveryOptions};
use crate::ResolveDocumentOutput;

mod runner;

pub struct WorkspaceCheckingOptions {
    pub discovery: WorkspaceDiscoveryOptions,
    pub max_snapshot_bytes: usize,
}

impl WorkspaceCheckingOptions {
    pub fn new(discovery: WorkspaceDiscoveryOptions) -> Self {
        Self {
            discovery,
            max_snapshot_bytes: 128 * 1024 * 1024,
        }
    }
}

pub struct WorkspaceCheckEvent {
    pub generation: u64,
    pub result: WorkspaceCheckResult,
}

pub enum WorkspaceCheckResult {
    Document {
        input: Box<DocumentInput>,
        result: Result<ResolveDocumentOutput, String>,
    },
    DiscoveryFailure(WorkspaceDiscoveryFailure),
    SnapshotLimit {
        uri: String,
        limit: usize,
    },
    DiscoverySettled,
    Settled,
}

#[derive(Default)]
struct Pending {
    /// A full replacement restarts discovery.
    options: Option<WorkspaceCheckingOptions>,
    /// The complete set of unsaved documents, applied without rediscovery.
    documents: Option<Vec<DocumentInput>>,
}

impl Pending {
    fn is_empty(&self) -> bool {
        self.options.is_none() && self.documents.is_none()
    }
}

struct Shared {
    pending: Mutex<Pending>,
    ready: Condvar,
    generation: AtomicU64,
    closed: AtomicBool,
    max_snapshot_bytes: AtomicUsize,
}

pub struct WorkspaceChecking {
    shared: Arc<Shared>,
    session: VersionLensSession,
}

impl VersionLensSession {
    /// Resolution tasks run on `runtime`, which the caller owns and keeps alive
    /// for as long as checking runs.
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
                documents: None,
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
    pub fn generation(&self) -> u64 {
        self.shared.generation.load(Ordering::Acquire)
    }

    /// Replaces pending input atomically. Consumers must compare each event's
    /// generation with the current generation before rendering or applying it.
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
            documents: None,
        };
        drop(pending);
        self.shared.ready.notify_one();
        Ok(previous + 1)
    }

    /// Replaces the unsaved documents without restarting discovery. Only
    /// documents whose text or version changed are checked again, and the
    /// generation stays the same.
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
mod tests;
