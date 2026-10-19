use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use versionlens_model::DocumentInput;

use super::{Shared, WorkspaceCheckEvent, WorkspaceCheckResult, WorkspaceCheckingOptions};
use crate::{
    CHECKING_CANCELLED, ResolveDocumentOutput, SessionTask, VersionLensSession, WorkspaceDiscovery,
    WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryOptions,
};

const MAX_ACTIVE: usize = 8;
const RETRY_DELAY: Duration = Duration::from_secs(1);
const MAX_RETRY_DELAY: Duration = Duration::from_mins(5);

/// The wait before the next check after `failures` failed checks in a row:
/// 1 s, doubling up to 5 min, so a broken registry is not polled every second.
pub(super) fn retry_delay(failures: u32) -> Duration {
    RETRY_DELAY
        .checked_mul(2_u32.saturating_pow(failures))
        .map_or(MAX_RETRY_DELAY, |delay| delay.min(MAX_RETRY_DELAY))
}

struct Entry {
    input: DocumentInput,
    due: Option<Instant>,
    task: Option<(u64, AbortOnDrop)>,
    /// Failed checks in a row, which set the retry backoff.
    failures: u32,
}

struct AbortOnDrop(tokio::task::AbortHandle);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

struct Completion {
    generation: u64,
    task_id: u64,
    uri: String,
    result: Result<ResolveDocumentOutput, String>,
}

pub(super) struct Runner<F> {
    session: VersionLensSession,
    shared: Arc<Shared>,
    on_event: F,
    generation: u64,
    cache_epoch: u64,
    discovery: Option<WorkspaceDiscovery>,
    discovery_options: Option<WorkspaceDiscoveryOptions>,
    discovery_overlays: Vec<PathBuf>,
    overlay_uris: BTreeSet<String>,
    discovery_retry: Option<Instant>,
    discovery_backoff: Duration,
    documents: BTreeMap<String, Entry>,
    snapshot_bytes: usize,
    max_snapshot_bytes: usize,
    active: usize,
    next_task: u64,
    settled: bool,
    next_prune: Instant,
    sender: Sender<Completion>,
    receiver: Receiver<Completion>,
    runtime: tokio::runtime::Handle,
}

impl<F: Fn(WorkspaceCheckEvent)> Runner<F> {
    pub(super) fn new(
        session: VersionLensSession,
        shared: Arc<Shared>,
        on_event: F,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        let (sender, receiver) = channel();
        Self {
            session,
            shared,
            on_event,
            generation: 0,
            cache_epoch: 0,
            discovery: None,
            discovery_options: None,
            discovery_overlays: Vec::new(),
            overlay_uris: BTreeSet::new(),
            discovery_retry: None,
            discovery_backoff: RETRY_DELAY,
            documents: BTreeMap::new(),
            snapshot_bytes: 0,
            max_snapshot_bytes: 0,
            active: 0,
            next_task: 0,
            settled: false,
            next_prune: Instant::now() + Duration::from_secs(30),
            sender,
            receiver,
            runtime,
        }
    }

    pub(super) fn run(mut self) {
        while !self.shared.closed.load(Ordering::Acquire) {
            self.apply_pending();
            if Instant::now() >= self.next_prune {
                self.session.purge_expired_cache();
                self.next_prune = Instant::now() + Duration::from_secs(30);
            }
            self.invalidate_expired_generation();
            self.complete();
            self.retry_discovery();
            self.discover();
            self.schedule();
            if !self.settled
                && self.discovery.is_none()
                && self.active == 0
                && self
                    .documents
                    .values()
                    .all(|entry| entry.due.is_none_or(|due| due > Instant::now()))
            {
                self.settled = true;
                self.emit(WorkspaceCheckResult::Settled);
            }
            self.wait();
        }
    }

    fn apply_pending(&mut self) {
        let (generation, pending) = {
            let mut pending = self
                .shared
                .pending
                .lock()
                .unwrap_or_else(crate::recover_poison);
            (
                self.shared.generation.load(Ordering::Acquire),
                std::mem::take(&mut *pending),
            )
        };
        if let Some(documents) = pending.documents {
            self.update_documents(documents);
        }
        let Some(WorkspaceCheckingOptions {
            discovery,
            max_snapshot_bytes,
        }) = pending.options
        else {
            return;
        };
        self.documents.clear();
        self.active = 0;
        self.snapshot_bytes = 0;
        self.max_snapshot_bytes = max_snapshot_bytes;
        self.generation = generation;
        self.settled = false;
        self.discovery = None;
        self.discovery_options = None;
        self.discovery_overlays.clear();
        self.overlay_uris = discovery
            .overlays
            .iter()
            .map(|input| input.uri.clone())
            .collect();
        self.discovery_retry = None;
        self.discovery_backoff = RETRY_DELAY;
        for input in &discovery.overlays {
            let bytes = snapshot_weight(input);
            if bytes > max_snapshot_bytes.saturating_sub(self.snapshot_bytes) {
                self.cache_epoch = self.current_cache_epoch();
                self.emit(WorkspaceCheckResult::SnapshotLimit {
                    uri: input.uri.clone(),
                    limit: max_snapshot_bytes,
                });
                return;
            }
            self.snapshot_bytes += bytes;
        }
        self.cache_epoch = self.current_cache_epoch();
        self.discovery_overlays = discovery
            .overlays
            .iter()
            .filter_map(|input| crate::workspace_path(&input.uri))
            .collect();
        let mut retry = WorkspaceDiscoveryOptions::new(discovery.roots.clone());
        retry.exclusions = discovery.exclusions.clone();
        retry.provider_exclusions = discovery.provider_exclusions.clone();
        retry.limits = discovery.limits;
        retry.cancellation = discovery.cancellation.clone();
        self.discovery_options = Some(retry);
        self.discovery = Some(self.session.discover_workspace_documents(discovery));
    }

    /// Applies a new set of unsaved documents in place. Changed documents are
    /// checked again, and closed ones fall back to their saved file.
    fn update_documents(&mut self, documents: Vec<DocumentInput>) {
        let uris = documents
            .iter()
            .map(|input| input.uri.clone())
            .collect::<BTreeSet<_>>();
        let closed = self
            .overlay_uris
            .difference(&uris)
            .cloned()
            .collect::<Vec<_>>();
        for uri in closed {
            let saved = self.documents.get(&uri).and_then(|entry| {
                let text = std::fs::read_to_string(crate::workspace_path(&uri)?).ok()?;
                Some(DocumentInput::new(
                    uri.clone(),
                    entry.input.language_id.clone(),
                    text,
                    entry.input.workspace_root.clone(),
                ))
            });
            self.remove(&uri);
            if let Some(saved) = saved {
                self.insert(saved);
            }
        }
        for input in documents {
            let unchanged = self.documents.get(&input.uri).is_some_and(|entry| {
                entry.input.version == input.version && entry.input.text == input.text
            });
            if unchanged {
                continue;
            }
            self.remove(&input.uri);
            self.insert(input);
        }
        self.discovery_overlays = uris
            .iter()
            .filter_map(|uri| crate::workspace_path(uri))
            .collect();
        if let Some(discovery) = self.discovery.as_mut() {
            discovery.skip_paths(&self.discovery_overlays);
        }
        self.overlay_uris = uris;
        self.settled = false;
    }

    fn remove(&mut self, uri: &str) {
        let Some(entry) = self.documents.remove(uri) else {
            return;
        };
        self.snapshot_bytes = self
            .snapshot_bytes
            .saturating_sub(snapshot_weight(&entry.input));
        if entry.task.is_some() {
            self.active = self.active.saturating_sub(1);
        }
    }

    fn current_cache_epoch(&self) -> u64 {
        self.session
            .storage_state
            .cache_epoch
            .load(Ordering::Acquire)
    }

    fn invalidate_expired_generation(&mut self) {
        let epoch = self.current_cache_epoch();
        if epoch == self.cache_epoch {
            return;
        }
        self.cache_epoch = epoch;
        self.active = 0;
        self.settled = false;
        for entry in self.documents.values_mut() {
            entry.task = None;
            entry.due = Some(Instant::now());
        }
    }

    fn discover(&mut self) {
        for _ in 0..32 {
            if !self.current() {
                return;
            }
            let Some(discovery) = self.discovery.as_mut() else {
                return;
            };
            match discovery.next() {
                Some(Ok(input)) => self.insert(input),
                Some(Err(failure)) => {
                    if matches!(
                        failure.kind,
                        WorkspaceDiscoveryFailureKind::Io { .. }
                            | WorkspaceDiscoveryFailureKind::InvalidUtf8
                    ) {
                        self.discovery_retry
                            .get_or_insert_with(|| Instant::now() + self.discovery_backoff);
                    }
                    self.emit(WorkspaceCheckResult::DiscoveryFailure(failure));
                }
                None => {
                    self.discovery = None;
                    self.emit(WorkspaceCheckResult::DiscoverySettled);
                    return;
                }
            }
        }
    }

    fn retry_discovery(&mut self) {
        if !self.current()
            || self.discovery.is_some()
            || self.discovery_retry.is_none_or(|due| due > Instant::now())
        {
            return;
        }
        self.discovery_retry = None;
        self.discovery_backoff = (self.discovery_backoff * 2).min(MAX_RETRY_DELAY);
        if let Some(options) = &self.discovery_options {
            let mut discovery = self.session.discover_workspace_documents(options.clone());
            discovery.skip_paths(&self.discovery_overlays);
            self.discovery = Some(discovery);
            self.settled = false;
        }
    }

    fn insert(&mut self, input: DocumentInput) {
        if self.documents.contains_key(&input.uri) {
            return;
        }
        let bytes = snapshot_weight(&input);
        if bytes > self.max_snapshot_bytes.saturating_sub(self.snapshot_bytes) {
            self.emit(WorkspaceCheckResult::SnapshotLimit {
                uri: input.uri,
                limit: self.max_snapshot_bytes,
            });
            return;
        }
        self.snapshot_bytes += bytes;
        self.documents.insert(
            input.uri.clone(),
            Entry {
                input,
                due: Some(Instant::now()),
                task: None,
                failures: 0,
            },
        );
    }

    fn schedule(&mut self) {
        if !self.current() {
            return;
        }
        let now = Instant::now();
        let uris = self
            .documents
            .iter()
            .filter(|(_, entry)| entry.task.is_none() && entry.due.is_some_and(|due| due <= now))
            .take(MAX_ACTIVE.saturating_sub(self.active))
            .map(|(uri, _)| uri.clone())
            .collect::<Vec<_>>();
        for uri in uris {
            self.submit(uri);
        }
    }

    fn submit(&mut self, uri: String) {
        let Some(task_id) = self.next_task.checked_add(1) else {
            self.shared.closed.store(true, Ordering::Release);
            return;
        };
        self.next_task = task_id;
        let Some(entry) = self.documents.get_mut(&uri) else {
            return;
        };
        let sender = self.sender.clone();
        let shared = Arc::clone(&self.shared);
        let generation = self.generation;
        let completion_uri = uri.clone();
        let session = self.session.clone();
        let input = entry.input.clone();
        let run = self
            .runtime
            .spawn(async move { session.run_task(SessionTask::Background(input)).await });
        let abort = run.abort_handle();
        self.runtime.spawn(async move {
            let result = match run.await {
                Ok(result) => result,
                Err(error) if error.is_panic() => Err("workspace check task panicked".to_owned()),
                Err(_) => return,
            };
            if !shared.closed.load(Ordering::Acquire)
                && shared.generation.load(Ordering::Acquire) == generation
            {
                let _ = sender.send(Completion {
                    generation,
                    task_id,
                    uri: completion_uri,
                    result,
                });
                shared.ready.notify_one();
            }
        });
        entry.task = Some((task_id, AbortOnDrop(abort)));
        self.active += 1;
    }

    fn complete(&mut self) {
        while let Ok(completion) = self.receiver.try_recv() {
            if completion.generation != self.generation || !self.current() {
                continue;
            }
            let Some(entry) = self.documents.get_mut(&completion.uri) else {
                continue;
            };
            if entry
                .task
                .as_ref()
                .is_none_or(|(id, _)| *id != completion.task_id)
            {
                continue;
            }
            entry.task = None;
            self.active = self.active.saturating_sub(1);
            let delay = match &completion.result {
                Ok(_) => {
                    entry.failures = 0;
                    self.session.document_check_delay(&entry.input)
                }
                // A cancelled check says nothing about the registry, so it
                // does not grow the backoff.
                Err(error) if error == CHECKING_CANCELLED => Some(RETRY_DELAY),
                Err(_) => {
                    let delay = retry_delay(entry.failures);
                    entry.failures = entry.failures.saturating_add(1);
                    Some(delay)
                }
            };
            entry.due = delay.map(|delay| Instant::now() + delay.max(RETRY_DELAY));
            let input = Box::new(entry.input.clone());
            self.emit(WorkspaceCheckResult::Document {
                input,
                result: completion.result,
            });
        }
    }

    fn current(&self) -> bool {
        !self.shared.closed.load(Ordering::Acquire)
            && self.shared.generation.load(Ordering::Acquire) == self.generation
    }

    fn emit(&self, result: WorkspaceCheckResult) {
        if self.current() {
            (self.on_event)(WorkspaceCheckEvent {
                generation: self.generation,
                result,
            });
        }
    }

    fn wait(&self) {
        let pending = self
            .shared
            .pending
            .lock()
            .unwrap_or_else(crate::recover_poison);
        if pending.is_empty() && !self.shared.closed.load(Ordering::Acquire) {
            drop(
                self.shared
                    .ready
                    .wait_timeout(pending, Duration::from_millis(100))
                    .unwrap_or_else(crate::recover_poison),
            );
        }
    }
}

pub(super) fn snapshot_weight(input: &DocumentInput) -> usize {
    input
        .text
        .capacity()
        .saturating_add(input.uri.capacity().saturating_mul(2))
        .saturating_add(input.language_id.capacity())
        .saturating_add(input.workspace_root.as_ref().map_or(0, String::capacity))
        .saturating_add(size_of::<Entry>() + 128)
}
