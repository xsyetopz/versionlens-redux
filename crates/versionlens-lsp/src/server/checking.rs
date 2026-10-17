use std::collections::{HashMap, HashSet};
use std::mem;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use lsp_server::{Connection, Message, Notification, Request, RequestId};
use lsp_types::notification::Notification as _;
use lsp_types::{CodeLens, Diagnostic, MessageType};
use serde_json::Value;
use tokio::runtime::Handle;
use versionlens_core::{
    AnalyzeDocumentOutput, CHECKING_CANCELLED, WorkspaceCheckEvent, WorkspaceCheckResult,
    WorkspaceChecking,
};
use versionlens_model::DocumentInput;

use super::publish_diagnostics;
use crate::protocol::{DocumentStatus, StatusNotification, StatusParams};
use crate::state::{CheckedDocument, ResolvedDocument, VersionLensLspState, resolved_document};

const EVENT_QUEUE_CAPACITY: usize = 128;

/// How long edits must pause before the edited documents are checked again.
pub(super) const EDIT_DEBOUNCE: Duration = Duration::from_millis(300);

/// The analysis of one checked document, made on the runtime because it
/// reads the persistent cache.
pub(super) struct Analyzed {
    generation: u64,
    document: CheckedDocument,
    analysis: Result<AnalyzeDocumentOutput, String>,
}

/// Documents waiting for their debounce before they are sent to checking.
/// Each document has its own deadline, so steady edits to one document do
/// not hold back another.
#[derive(Default)]
struct PendingDocuments {
    due: HashMap<String, Instant>,
    /// The text last sent to checking for each open document. A document
    /// still in its debounce is sent with this text, so a check triggered by
    /// another document does not resolve a half-typed edit.
    submitted: HashMap<String, DocumentInput>,
}

struct PublishedDocument {
    diagnostics: Vec<Diagnostic>,
    code_lenses: Vec<CodeLens>,
}

struct DiscoveryState {
    full_scan_pending: bool,
    exhausted: bool,
}

pub(super) struct WorkspaceController {
    checking: WorkspaceChecking,
    runtime: Handle,
    shutdown: Sender<()>,
    pub(super) receiver: Receiver<WorkspaceCheckEvent>,
    published: HashMap<String, PublishedDocument>,
    seen_documents: HashSet<String>,
    discovery: DiscoveryState,
    reported_failures: HashSet<String>,
    next_request: u64,
    pending: PendingDocuments,
    statuses: HashMap<String, DocumentStatus>,
    analyzed_sender: Sender<Analyzed>,
    pub(super) analyzed: Receiver<Analyzed>,
}

impl WorkspaceController {
    pub(super) fn start(state: &VersionLensLspState, runtime: Handle) -> std::io::Result<Self> {
        let (sender, receiver) = bounded(EVENT_QUEUE_CAPACITY);
        let (shutdown, stopped) = bounded(1);
        let checking = state.session.start_workspace_checking(
            state.workspace_checking_options(),
            runtime.clone(),
            move |event| {
                crossbeam_channel::select! {
                    send(sender, event) -> _ => {}
                    recv(stopped) -> _ => {}
                }
            },
        )?;
        let (analyzed_sender, analyzed) = unbounded();
        Ok(Self {
            checking,
            runtime,
            shutdown,
            receiver,
            published: HashMap::new(),
            seen_documents: HashSet::new(),
            discovery: DiscoveryState {
                full_scan_pending: true,
                exhausted: false,
            },
            reported_failures: HashSet::new(),
            next_request: 0,
            pending: PendingDocuments::default(),
            statuses: HashMap::new(),
            analyzed_sender,
            analyzed,
        })
    }

    fn stop(&self) {
        let _ = self.shutdown.try_send(());
        self.checking.stop();
    }

    pub(super) fn replace(&mut self, state: &VersionLensLspState) -> Result<()> {
        self.checking
            .replace(state.workspace_checking_options())
            .map_err(anyhow::Error::msg)?;
        self.pending.submitted.clear();
        self.seen_documents.clear();
        self.discovery.full_scan_pending = true;
        self.discovery.exhausted = false;
        Ok(())
    }

    /// Sends the open documents to checking without restarting discovery.
    pub(super) fn update_documents(&mut self, state: &VersionLensLspState) -> Result<()> {
        let overlays = self.overlays(state);
        self.checking
            .update_documents(overlays)
            .map_err(anyhow::Error::msg)
    }

    /// The open documents to check: the current text, or the last sent text
    /// for documents still in their debounce.
    fn overlays(&mut self, state: &VersionLensLspState) -> Vec<DocumentInput> {
        let mut uris = state.document_uris();
        uris.sort();
        uris.into_iter()
            .filter_map(|uri| {
                let held = self
                    .pending
                    .due
                    .contains_key(&uri)
                    .then(|| self.pending.submitted.get(&uri).cloned())
                    .flatten();
                let input = held.or_else(|| state.document_work(&uri).map(|work| work.input))?;
                self.pending.submitted.insert(uri, input.clone());
                Some(input)
            })
            .collect()
    }

    pub(super) fn restart(&mut self, state: &VersionLensLspState) -> std::io::Result<()> {
        self.stop();
        let mut replacement = Self::start(state, self.runtime.clone())?;
        replacement.published = mem::take(&mut self.published);
        replacement.reported_failures = mem::take(&mut self.reported_failures);
        replacement.next_request = self.next_request;
        replacement.pending.due = mem::take(&mut self.pending.due);
        replacement.statuses = mem::take(&mut self.statuses);
        mem::swap(self, &mut replacement);
        Ok(())
    }

    /// Queues an opened or edited document for checking at `due`. Edits
    /// push its deadline back, so a burst of edits is checked once.
    pub(super) fn schedule_document(&mut self, uri: &str, due: Instant, debounced: bool) {
        // The next result is published even when it equals the last one, so
        // the client sees diagnostics for the new document version.
        self.published.remove(uri);
        let deadline = self.pending.due.entry(uri.to_owned()).or_insert(due);
        *deadline = if debounced { due } else { (*deadline).min(due) };
    }

    pub(super) fn next_due(&self) -> Option<Instant> {
        self.pending.due.values().min().copied()
    }

    /// Sends the queued documents to checking once their debounce has passed.
    pub(super) fn flush_due(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        now: Instant,
    ) -> Result<()> {
        let due = self
            .pending
            .due
            .iter()
            .filter(|(_, due)| **due <= now)
            .map(|(uri, _)| uri.clone())
            .collect::<Vec<_>>();
        if due.is_empty() {
            return Ok(());
        }
        for uri in &due {
            self.pending.due.remove(uri);
        }
        self.update_documents(state)?;
        for uri in due {
            if state.document_is_open(&uri) {
                self.set_status(connection, &uri, DocumentStatus::Busy)?;
            }
        }
        Ok(())
    }

    pub(super) fn forget_document(&mut self, uri: &str) {
        self.published.remove(uri);
        self.seen_documents.remove(uri);
        self.pending.due.remove(uri);
        self.pending.submitted.remove(uri);
        self.statuses.remove(uri);
    }

    fn set_status(
        &mut self,
        connection: &Connection,
        uri: &str,
        status: DocumentStatus,
    ) -> Result<()> {
        let previous = self
            .statuses
            .insert(uri.to_owned(), status)
            .unwrap_or(DocumentStatus::Idle);
        if previous == status {
            return Ok(());
        }
        connection
            .sender
            .send(Message::Notification(Notification {
                method: StatusNotification::METHOD.to_owned(),
                params: serde_json::to_value(StatusParams {
                    uri: uri.parse()?,
                    status,
                })?,
            }))
            .context("failed to send document status")
    }

    /// Publishes an analysis if its document has not changed since checking.
    pub(super) fn finish_analysis(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        analyzed: Analyzed,
    ) -> Result<()> {
        if analyzed.generation != self.checking.generation() {
            return Ok(());
        }
        let Some(document) = state.checked_document(&analyzed.document.input) else {
            return Ok(());
        };
        let open = document.open;
        let uri = document.uri.clone();
        let status = match analyzed.analysis {
            Ok(analysis) => {
                let resolved = resolved_document(analysis, &document.uri, document.generation);
                self.publish_result(state, connection, document, resolved)?;
                DocumentStatus::Idle
            }
            // A failure is about VersionLens or the registry, not the
            // manifest, so it goes to the log and the status and the last
            // published result stays.
            Err(message) => {
                log_message(
                    connection,
                    MessageType::ERROR,
                    format!("VersionLens could not check {uri}: {message}"),
                )?;
                DocumentStatus::Error
            }
        };
        if open {
            // The result is for the current text, so a pending check of the
            // same text would be skipped by checking and never clear `busy`.
            self.pending.due.remove(&uri);
            self.set_status(connection, &uri, status)?;
        }
        Ok(())
    }

    pub(super) fn handle(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        event: WorkspaceCheckEvent,
    ) -> Result<()> {
        if event.generation != self.checking.generation() {
            return Ok(());
        }
        match event.result {
            WorkspaceCheckResult::Document { input, result } => {
                let Some(document) = state.checked_document(&input) else {
                    return Ok(());
                };
                let generation = self.checking.generation();
                let Err(message) = result else {
                    self.analyze(state, document, generation);
                    return Ok(());
                };
                // A newer check of the same document follows.
                if message == CHECKING_CANCELLED {
                    return Ok(());
                }
                self.finish_analysis(
                    state,
                    connection,
                    Analyzed {
                        generation,
                        document,
                        analysis: Err(message),
                    },
                )
            }
            WorkspaceCheckResult::SnapshotLimit { uri, limit } => {
                // Checking stops at the first document over the limit, so no
                // busy document gets a result.
                let busy = self
                    .statuses
                    .iter()
                    .filter(|(_, status)| **status == DocumentStatus::Busy)
                    .map(|(uri, _)| uri.clone())
                    .collect::<Vec<_>>();
                for target in busy {
                    self.set_status(connection, &target, DocumentStatus::Error)?;
                }
                log_message(
                    connection,
                    MessageType::WARNING,
                    format!(
                        "VersionLens stopped checking at {uri}: the workspace snapshot limit of {limit} bytes was exceeded"
                    ),
                )
            }
            WorkspaceCheckResult::DiscoveryFailure(failure) => {
                let message = failure.to_string();
                // Discovery retries with backoff, so the same failure would
                // otherwise be logged on every retry.
                if !self.reported_failures.insert(message.clone()) {
                    return Ok(());
                }
                log_message(
                    connection,
                    MessageType::WARNING,
                    format!("VersionLens workspace discovery: {message}"),
                )
            }
            WorkspaceCheckResult::DiscoverySettled => {
                self.finish_discovery();
                Ok(())
            }
            WorkspaceCheckResult::Settled => self.finish_generation(state, connection),
        }
    }

    fn analyze(&self, state: &VersionLensLspState, document: CheckedDocument, generation: u64) {
        let session = state.session.clone();
        let sender = self.analyzed_sender.clone();
        self.runtime.spawn_blocking(move || {
            let input = document.input.clone();
            let analysis = catch_unwind(AssertUnwindSafe(|| session.analyze_document(input)))
                .map_err(|_| "VersionLens analysis panicked".to_owned());
            let _ = sender.send(Analyzed {
                generation,
                document,
                analysis,
            });
        });
    }

    fn publish_result(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        document: CheckedDocument,
        resolved: ResolvedDocument,
    ) -> Result<()> {
        self.seen_documents.insert(document.uri.clone());
        let diagnostics_changed = self
            .published
            .get(&document.uri)
            .is_none_or(|previous| previous.diagnostics != resolved.diagnostics);
        let lenses_changed = document.open
            && self.published.get(&document.uri).map_or_else(
                || !resolved.code_lenses.is_empty(),
                |previous| previous.code_lenses != resolved.code_lenses,
            );
        if diagnostics_changed {
            publish_diagnostics(
                connection,
                document.uri.parse()?,
                resolved.diagnostics.clone(),
                document.version,
            )?;
        }
        if lenses_changed && state.client.code_lens_refresh {
            self.request_lens_refresh(connection)?;
        }
        self.published.insert(
            document.uri,
            PublishedDocument {
                diagnostics: resolved.diagnostics,
                code_lenses: resolved.code_lenses,
            },
        );
        Ok(())
    }

    fn finish_generation(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
    ) -> Result<()> {
        if !self.discovery.full_scan_pending || !self.discovery.exhausted {
            return Ok(());
        }
        let removed = self
            .published
            .iter()
            .filter(|(uri, _)| !self.seen_documents.contains(*uri) && !state.document_is_open(uri))
            .map(|(uri, _)| uri.clone())
            .collect::<Vec<_>>();
        for uri in removed {
            publish_diagnostics(connection, uri.parse()?, Vec::new(), None)?;
            self.published.remove(&uri);
        }
        self.seen_documents.clear();
        self.discovery.full_scan_pending = false;
        self.discovery.exhausted = false;
        Ok(())
    }

    fn finish_discovery(&mut self) {
        self.discovery.exhausted = true;
    }

    fn request_lens_refresh(&mut self, connection: &Connection) -> Result<()> {
        self.next_request = self
            .next_request
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("code lens refresh request ID exhausted"))?;
        connection
            .sender
            .send(Message::Request(Request {
                id: RequestId::from(format!(
                    "versionlens/workspace-refresh/{}",
                    self.next_request
                )),
                method: "workspace/codeLens/refresh".to_owned(),
                params: Value::Null,
            }))
            .context("failed to request code lens refresh")
    }
}

impl Drop for WorkspaceController {
    fn drop(&mut self) {
        self.stop();
    }
}

fn log_message(connection: &Connection, kind: MessageType, message: String) -> Result<()> {
    connection
        .sender
        .send(Message::Notification(Notification {
            method: "window/logMessage".to_owned(),
            params: serde_json::to_value(lsp_types::LogMessageParams { typ: kind, message })?,
        }))
        .context("failed to send log message")
}

#[cfg(test)]
mod tests;
