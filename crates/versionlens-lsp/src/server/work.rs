use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender, unbounded};
use lsp_server::{Connection, ErrorCode, Message, Request, RequestId, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::runtime::Runtime;
use tokio::task::AbortHandle;
use versionlens_core::{ResolveDocumentOutput, SessionTask};

use super::{publish_diagnostics, respond, respond_error};
use crate::state::{DocumentWork, ResolvedDocument, VersionLensLspState};

pub(super) struct Completion {
    id: RequestId,
    output: Result<ResolveDocumentOutput, String>,
}

enum RequestKind {
    Lenses,
    Update,
}

struct Pending {
    work: DocumentWork,
    cancellation: AbortHandle,
    kind: RequestKind,
}

pub(super) struct WorkQueue {
    pending: HashMap<RequestId, Pending>,
    completed: HashMap<String, CompletedDocument>,
    sender: Sender<Completion>,
    pub(super) receiver: Receiver<Completion>,
    applications: HashMap<RequestId, PendingApplication>,
    next_id: u64,
    runtime: Runtime,
}

struct CompletedDocument {
    generation: u64,
    resolved: ResolvedDocument,
}

struct PendingApplication {
    original: RequestId,
    deadline: Instant,
}

struct Submission {
    id: RequestId,
    work: DocumentWork,
    task: SessionTask,
    kind: RequestKind,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateArguments {
    uri: String,
    generation: u64,
    arguments: Vec<String>,
}

impl WorkQueue {
    pub(super) fn new() -> Result<Self> {
        let (sender, receiver) = unbounded();
        Ok(Self {
            pending: HashMap::new(),
            completed: HashMap::new(),
            sender,
            receiver,
            applications: HashMap::new(),
            next_id: 0,
            runtime: Runtime::new().context("failed to create LSP async runtime")?,
        })
    }

    pub(super) fn lenses(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        id: RequestId,
        uri: &str,
    ) -> Result<()> {
        let Some(work) = state.document_work(uri) else {
            return respond(connection, id, json!([]));
        };
        if let Some(completed) = self
            .completed
            .get(uri)
            .filter(|completed| completed.generation == work.generation)
        {
            return respond_with_document(connection, id, &work, &completed.resolved);
        }
        if state.session.document_is_fresh(&work.input) {
            let resolved = state.analyzed_document(uri);
            if let Some(resolved) = resolved {
                let result = respond_with_document(connection, id, &work, &resolved);
                self.completed.insert(
                    uri.to_owned(),
                    CompletedDocument {
                        generation: work.generation,
                        resolved,
                    },
                );
                return result;
            }
            return respond(connection, id, json!([]));
        }
        let task = SessionTask::Resolve(work.input.clone());
        self.submit(
            state,
            connection,
            Submission {
                id,
                work,
                task,
                kind: RequestKind::Lenses,
            },
        )
    }

    pub(super) fn update(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        id: RequestId,
        arguments: Vec<Value>,
    ) -> Result<()> {
        if !state.client.workspace_edits {
            return respond_error(
                connection,
                id,
                ErrorCode::RequestFailed,
                "editor does not support versioned workspace edits".to_owned(),
            );
        }
        let parsed = if arguments.len() == 1 {
            serde_json::from_value::<UpdateArguments>(arguments[0].clone()).ok()
        } else {
            None
        };
        let Some(args) = parsed else {
            return respond_error(
                connection,
                id,
                ErrorCode::InvalidParams,
                "invalid update arguments".to_owned(),
            );
        };
        let Some(work) = state
            .document_work(&args.uri)
            .filter(|work| work.generation == args.generation)
        else {
            return respond_error(
                connection,
                id,
                ErrorCode::ContentModified,
                "document changed; request fresh code lenses".to_owned(),
            );
        };
        if !(2..=4).contains(&args.arguments.len()) {
            return respond_error(
                connection,
                id,
                ErrorCode::InvalidParams,
                "invalid dependency arguments".to_owned(),
            );
        }
        let task = SessionTask::Command {
            document: work.input.clone(),
            command: args.arguments.get(2).cloned(),
            dependency: args.arguments.get(1).cloned(),
            selected_version: args.arguments.get(3).cloned(),
        };
        self.submit(
            state,
            connection,
            Submission {
                id,
                work,
                task,
                kind: RequestKind::Update,
            },
        )
    }

    fn submit(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        submission: Submission,
    ) -> Result<()> {
        let Submission {
            id,
            work,
            task,
            kind,
        } = submission;
        if self.pending.contains_key(&id) {
            return respond_error(
                connection,
                id,
                ErrorCode::InvalidRequest,
                "request ID is already active".to_owned(),
            );
        }
        let sender = self.sender.clone();
        let completion_id = id.clone();
        let session = state.session.clone();
        let handle = self.runtime.spawn(async move {
            let output = session.run_task(task).await;
            let _ = sender.send(Completion {
                id: completion_id,
                output,
            });
        });
        self.pending.insert(
            id,
            Pending {
                work,
                cancellation: handle.abort_handle(),
                kind,
            },
        );
        Ok(())
    }

    pub(super) fn complete(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        completion: Completion,
    ) -> Result<()> {
        let Some(pending) = self.pending.remove(&completion.id) else {
            return Ok(());
        };
        if !state.work_is_current(&pending.work) {
            return respond_error(
                connection,
                completion.id,
                ErrorCode::ContentModified,
                "document changed during checking".to_owned(),
            );
        }
        let output = match completion.output {
            Ok(output) => output,
            Err(message) => {
                return respond_error(connection, completion.id, ErrorCode::RequestFailed, message);
            }
        };
        match pending.kind {
            RequestKind::Lenses => {
                let resolved = state.analyzed_document(&pending.work.uri);
                if let Some(resolved) = resolved {
                    let result =
                        respond_with_document(connection, completion.id, &pending.work, &resolved);
                    self.completed.insert(
                        pending.work.uri,
                        CompletedDocument {
                            generation: pending.work.generation,
                            resolved,
                        },
                    );
                    return result;
                }
                respond(connection, completion.id, json!([]))
            }
            RequestKind::Update => self.apply(state, connection, completion.id, output),
        }
    }

    fn apply(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        id: RequestId,
        output: ResolveDocumentOutput,
    ) -> Result<()> {
        let Some(plan) = output.edit_plan.filter(|plan| {
            plan.documents
                .iter()
                .any(|document| !document.edits.is_empty())
        }) else {
            return respond_error(
                connection,
                id,
                ErrorCode::RequestFailed,
                "no applicable update is available".to_owned(),
            );
        };
        let edit = match state.workspace_edit(&plan) {
            Ok(edit) => edit,
            Err(message) => {
                return respond_error(connection, id, ErrorCode::ContentModified, message);
            }
        };
        self.next_id += 1;
        let application_id = RequestId::from(format!("versionlens/apply/{}", self.next_id));
        self.applications.insert(
            application_id.clone(),
            PendingApplication {
                original: id,
                deadline: Instant::now() + Duration::from_secs(30),
            },
        );
        connection
            .sender
            .send(Message::Request(Request {
                id: application_id,
                method: "workspace/applyEdit".to_owned(),
                params: json!({"label":"Update dependency", "edit":edit}),
            }))
            .context("failed to send workspace edit")
    }

    pub(super) fn applied(&mut self, connection: &Connection, response: Response) -> Result<()> {
        let Some(application) = self.applications.remove(&response.id) else {
            return Ok(());
        };
        let original = application.original;
        match response.response_result {
            Ok(value) if value.get("applied").and_then(Value::as_bool) == Some(true) => {
                respond(connection, original, Value::Null)
            }
            Ok(value) => respond_error(
                connection,
                original,
                ErrorCode::RequestFailed,
                value
                    .get("failureReason")
                    .and_then(Value::as_str)
                    .unwrap_or("editor rejected workspace edit")
                    .to_owned(),
            ),
            Err(error) => respond_error(
                connection,
                original,
                ErrorCode::RequestFailed,
                error.message,
            ),
        }
    }

    pub(super) fn cancel(&mut self, connection: &Connection, id: &RequestId) -> Result<()> {
        if let Some(pending) = self.pending.remove(id) {
            pending.cancellation.abort();
            respond_error(
                connection,
                id.clone(),
                ErrorCode::RequestCanceled,
                "request cancelled".to_owned(),
            )?;
        }
        if let Some(application) = self
            .applications
            .iter()
            .find(|(_, application)| &application.original == id)
            .map(|(id, _)| id.clone())
        {
            self.finish_application(
                connection,
                &application,
                ErrorCode::RequestCanceled,
                "request cancelled",
            )?;
        }
        Ok(())
    }

    pub(super) fn expire_applications(
        &mut self,
        connection: &Connection,
        now: std::time::Instant,
    ) -> Result<()> {
        let expired = self
            .applications
            .iter()
            .filter(|(_, application)| application.deadline <= now)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in expired {
            self.finish_application(
                connection,
                &id,
                ErrorCode::RequestFailed,
                "editor did not confirm the workspace edit before the deadline",
            )?;
        }
        Ok(())
    }

    fn finish_application(
        &mut self,
        connection: &Connection,
        id: &RequestId,
        code: ErrorCode,
        message: &str,
    ) -> Result<()> {
        let Some(application) = self.applications.remove(id) else {
            return Ok(());
        };
        connection
            .sender
            .send(Message::Notification(lsp_server::Notification::new(
                "$/cancelRequest".to_owned(),
                json!({"id":id}),
            )))?;
        respond_error(connection, application.original, code, message.to_owned())
    }

    pub(super) fn invalidate(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
    ) -> Result<()> {
        self.completed.retain(|uri, completed| {
            state
                .document_work(uri)
                .is_some_and(|work| work.generation == completed.generation)
        });
        let ids = self
            .pending
            .iter()
            .filter(|(_, pending)| !state.work_is_current(&pending.work))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in ids {
            let Some(pending) = self.pending.remove(&id) else {
                continue;
            };
            pending.cancellation.abort();
            respond_error(
                connection,
                id,
                ErrorCode::ContentModified,
                "document changed during checking".to_owned(),
            )?;
        }
        Ok(())
    }
}

fn respond_with_document(
    connection: &Connection,
    id: RequestId,
    work: &DocumentWork,
    resolved: &ResolvedDocument,
) -> Result<()> {
    respond(
        connection,
        id,
        serde_json::to_value(resolved.code_lenses.as_slice())?,
    )?;
    publish_diagnostics(
        connection,
        work.uri.parse()?,
        resolved.diagnostics.clone(),
        work.version,
    )
}

#[cfg(test)]
mod tests;
