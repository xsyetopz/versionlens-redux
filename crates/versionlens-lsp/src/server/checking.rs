use std::collections::{HashMap, HashSet};
use std::mem;

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender, bounded};
use lsp_server::{Connection, Message, Notification, Request, RequestId};
use lsp_types::{CodeLens, Diagnostic, DiagnosticSeverity};
use serde_json::Value;
use versionlens_core::{WorkspaceCheckEvent, WorkspaceCheckResult, WorkspaceChecking};

use super::publish_diagnostics;
use crate::state::{ResolvedDocument, VersionLensLspState};

const EVENT_QUEUE_CAPACITY: usize = 128;

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
    shutdown: Sender<()>,
    pub(super) receiver: Receiver<WorkspaceCheckEvent>,
    published: HashMap<String, PublishedDocument>,
    seen_documents: HashSet<String>,
    discovery: DiscoveryState,
    reported_failures: HashSet<String>,
    next_request: u64,
}

impl WorkspaceController {
    pub(super) fn start(state: &VersionLensLspState) -> std::io::Result<Self> {
        let (sender, receiver) = bounded(EVENT_QUEUE_CAPACITY);
        let (shutdown, stopped) = bounded(1);
        let checking = state.session.start_workspace_checking(
            state.workspace_checking_options(),
            move |event| {
                crossbeam_channel::select! {
                    send(sender, event) -> _ => {}
                    recv(stopped) -> _ => {}
                }
            },
        )?;
        Ok(Self {
            checking,
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

    pub(super) fn restart(&mut self, state: &VersionLensLspState) -> std::io::Result<()> {
        self.stop();
        let mut replacement = Self::start(state)?;
        replacement.published = mem::take(&mut self.published);
        replacement.discovery.failures = mem::take(&mut self.discovery.failures);
        replacement.reported_failures = mem::take(&mut self.reported_failures);
        replacement.next_request = self.next_request;
        mem::swap(self, &mut replacement);
        Ok(())
    }

    pub(super) fn observe_open_document(&mut self, uri: &str, diagnostics: Vec<Diagnostic>) {
        self.discovery.failures.remove(uri);
        self.discovery.pending_failures.remove(uri);
        self.published.insert(
            uri.to_owned(),
            PublishedDocument {
                diagnostics,
                code_lenses: Vec::new(),
                discovery_failure: false,
            },
        );
    }

    pub(super) fn forget_document(&mut self, uri: &str) {
        self.published.remove(uri);
        self.seen_documents.remove(uri);
        self.discovery.failures.remove(uri);
        self.discovery.pending_failures.remove(uri);
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
                let resolved = match result {
                    Ok(_) => state.analyzed_input(
                        document.input.clone(),
                        &document.uri,
                        document.generation,
                    ),
                    Err(message) => ResolvedDocument {
                        code_lenses: Vec::new(),
                        diagnostics: vec![warning(message)],
                    },
                };
                self.publish_result(state, connection, document, resolved)
            }
            WorkspaceCheckResult::SnapshotLimit { uri, limit } => self.publish_discovery_warning(
                state,
                connection,
                &uri,
                format!("workspace snapshot limit of {limit} bytes was exceeded"),
            ),
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
        document: crate::state::CheckedDocument,
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
