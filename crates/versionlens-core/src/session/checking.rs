use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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

struct Shared {
    pending: Mutex<Option<WorkspaceCheckingOptions>>,
    ready: Condvar,
    generation: AtomicU64,
    closed: AtomicBool,
}

pub struct WorkspaceChecking {
    shared: Arc<Shared>,
    session: VersionLensSession,
}

impl VersionLensSession {
    pub fn start_workspace_checking(
        &self,
        options: WorkspaceCheckingOptions,
        on_event: impl Fn(WorkspaceCheckEvent) + Send + 'static,
    ) -> std::io::Result<WorkspaceChecking> {
        establish_snapshot(self, &options, true);
        let shared = Arc::new(Shared {
            pending: Mutex::new(Some(options)),
            ready: Condvar::new(),
            generation: AtomicU64::new(1),
            closed: AtomicBool::new(false),
        });
        let session = self.clone();
        let worker_shared = Arc::clone(&shared);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
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
            .unwrap_or_else(crate::recover_poison);
        if self.shared.closed.load(Ordering::Acquire) {
            return Err("workspace checking is stopped");
        }
        let previous = self
            .shared
            .generation
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |generation| {
                generation.checked_add(1)
            })
            .map_err(|_| "workspace generation limit reached")?;
        establish_snapshot(&self.session, &options, false);
        *pending = Some(options);
        drop(pending);
        self.shared.ready.notify_one();
        Ok(previous + 1)
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
    let bytes = options
        .discovery
        .overlays
        .iter()
        .fold(0_usize, |total, input| {
            total.saturating_add(runner::snapshot_weight(input))
        });
    let documents = if bytes <= options.max_snapshot_bytes {
        options.discovery.overlays.clone()
    } else {
        Vec::new()
    };
    if !session.set_workspace_documents(documents) && !initial {
        session.invalidate_workspace();
    }
}

impl Drop for WorkspaceChecking {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests;
