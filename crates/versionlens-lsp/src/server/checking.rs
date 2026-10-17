use std::collections::{HashMap, HashSet};
use std::mem;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use lsp_server::{Connection, Message, Notification, Request, RequestId};
use lsp_types::notification::Notification as _;
use lsp_types::{CodeLens, Diagnostic, DiagnosticSeverity};
use serde_json::Value;
use tokio::runtime::Handle;
use versionlens_core::{
    AnalyzeDocumentOutput, WorkspaceCheckEvent, WorkspaceCheckResult, WorkspaceChecking,
};

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

/// Documents waiting for the debounce before they are sent to checking.
#[derive(Default)]
struct PendingDocuments {
    uris: HashSet<String>,
    due: Option<Instant>,
}

struct PublishedDocument {
    diagnostics: Vec<Diagnostic>,
    code_lenses: Vec<CodeLens>,
    discovery_failure: bool,
}

struct DiscoveryState {
    failures: HashSet<String>,
    pending_failures: HashSet<String>,
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
                failures: HashSet::new(),
                pending_failures: HashSet::new(),
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
        self.seen_documents.clear();
        self.discovery.pending_failures.clear();
        self.discovery.full_scan_pending = true;
        self.discovery.exhausted = false;
        Ok(())
    }

    /// Sends the open documents to checking without restarting discovery.
    pub(super) fn update_documents(&self, state: &VersionLensLspState) -> Result<()> {
        self.checking
            .update_documents(state.workspace_checking_options().discovery.overlays)
            .map_err(anyhow::Error::msg)
    }

    pub(super) fn restart(&mut self, state: &VersionLensLspState) -> std::io::Result<()> {
        self.stop();
        let mut replacement = Self::start(state, self.runtime.clone())?;
        replacement.published = mem::take(&mut self.published);
        replacement.discovery.failures = mem::take(&mut self.discovery.failures);
        replacement.reported_failures = mem::take(&mut self.reported_failures);
        replacement.next_request = self.next_request;
        replacement.pending = mem::take(&mut self.pending);
        replacement.statuses = mem::take(&mut self.statuses);
        mem::swap(self, &mut replacement);
        Ok(())
    }

    /// Queues an opened or edited document for checking at `due`. Edits
    /// push the deadline back, so a burst of edits is checked once.
    pub(super) fn schedule_document(&mut self, uri: &str, due: Instant, debounced: bool) {
        self.discovery.failures.remove(uri);
        self.discovery.pending_failures.remove(uri);
        // The next result is published even when it equals the last one, so
        // the client sees diagnostics for the new document version.
        self.published.remove(uri);
        self.pending.uris.insert(uri.to_owned());
        self.pending.due = Some(match self.pending.due {
            Some(current) if !debounced => current.min(due),
            _ => due,
        });
    }

    pub(super) fn next_due(&self) -> Option<Instant> {
        self.pending.due
    }

    /// Sends the queued documents to checking once their debounce has passed.
    pub(super) fn flush_due(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        now: Instant,
    ) -> Result<()> {
        if self.pending.due.is_none_or(|due| due > now) {
            return Ok(());
        }
        self.pending.due = None;
        self.update_documents(state)?;
        for uri in mem::take(&mut self.pending.uris) {
            if state.document_is_open(&uri) {
                self.set_status(connection, &uri, DocumentStatus::Busy)?;
            }
        }
        Ok(())
    }

    pub(super) fn forget_document(&mut self, uri: &str) {
        self.published.remove(uri);
        self.seen_documents.remove(uri);
        self.discovery.failures.remove(uri);
        self.discovery.pending_failures.remove(uri);
        self.pending.uris.remove(uri);
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
        let (resolved, status) = match analyzed.analysis {
            Ok(analysis) => (
                resolved_document(analysis, &document.uri, document.generation),
                DocumentStatus::Idle,
            ),
            Err(message) => (
                ResolvedDocument {
                    code_lenses: Vec::new(),
                    diagnostics: vec![warning(message)],
                },
                DocumentStatus::Error,
            ),
        };
        self.publish_result(state, connection, document, resolved)?;
        if open && !self.pending.uris.contains(&uri) {
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
                let (target, _) = state.diagnostic_target(&uri);
                if self.statuses.get(&target) == Some(&DocumentStatus::Busy) {
                    self.set_status(connection, &target, DocumentStatus::Error)?;
                }
                self.publish_discovery_warning(
                    state,
                    connection,
                    &uri,
                    format!("workspace snapshot limit of {limit} bytes was exceeded"),
                )
            }
            WorkspaceCheckResult::DiscoveryFailure(failure) => {
                let message = failure.to_string();
                if let Some(uri) = failure
                    .path
                    .as_deref()
                    .and_then(versionlens_core::workspace_file_uri)
                {
                    return self.publish_discovery_warning(state, connection, &uri, message);
                }
                if !self.reported_failures.insert(message.clone()) {
                    return Ok(());
                }
                connection
                    .sender
                    .send(Message::Notification(Notification {
                        method: "window/logMessage".to_owned(),
                        params: serde_json::json!({
                            "type": 2,
                            "message": format!("VersionLens workspace discovery: {message}"),
                        }),
                    }))
                    .context("failed to report workspace discovery failure")
            }
            WorkspaceCheckResult::DiscoverySettled => self.finish_discovery(connection),
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

    fn publish_discovery_warning(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        uri: &str,
        message: String,
    ) -> Result<()> {
        let (uri, version) = state.diagnostic_target(uri);
        self.discovery.failures.insert(uri.clone());
        self.discovery.pending_failures.insert(uri.clone());
        let diagnostics = vec![discovery_warning(message)];
        let changed = self
            .published
            .get(&uri)
            .is_none_or(|previous| previous.diagnostics != diagnostics);
        if changed {
            publish_diagnostics(connection, uri.parse()?, diagnostics.clone(), version)?;
        }
        self.published.insert(
            uri,
            PublishedDocument {
                diagnostics,
                code_lenses: Vec::new(),
                discovery_failure: true,
            },
        );
        Ok(())
    }

    fn publish_result(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        document: CheckedDocument,
        resolved: ResolvedDocument,
    ) -> Result<()> {
        self.seen_documents.insert(document.uri.clone());
        self.discovery.failures.remove(&document.uri);
        self.discovery.pending_failures.remove(&document.uri);
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
                discovery_failure: false,
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
            .filter(|(uri, published)| {
                !published.discovery_failure
                    && !self.seen_documents.contains(*uri)
                    && !state.document_is_open(uri)
            })
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

    fn finish_discovery(&mut self, connection: &Connection) -> Result<()> {
        let recovered = self
            .discovery
            .failures
            .difference(&self.discovery.pending_failures)
            .filter(|uri| {
                self.published
                    .get(*uri)
                    .is_some_and(|published| published.discovery_failure)
            })
            .cloned()
            .collect::<Vec<_>>();
        for uri in recovered {
            publish_diagnostics(connection, uri.parse()?, Vec::new(), None)?;
            self.published.remove(&uri);
        }
        self.discovery.failures = mem::take(&mut self.discovery.pending_failures);
        self.discovery.exhausted = true;
        Ok(())
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

fn warning(message: String) -> Diagnostic {
    Diagnostic {
        range: lsp_types::Range::default(),
        severity: Some(DiagnosticSeverity::WARNING),
        source: Some("VersionLens".to_owned()),
        message,
        ..Diagnostic::default()
    }
}

fn discovery_warning(message: String) -> Diagnostic {
    Diagnostic {
        source: Some("VersionLens workspace discovery".to_owned()),
        ..warning(message)
    }
}
