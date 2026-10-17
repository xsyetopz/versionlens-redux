use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender, unbounded};
use lsp_server::{Connection, ErrorCode, Message, Request, RequestId, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::runtime::Handle;
use tokio::task::AbortHandle;
use versionlens_core::{AnalyzeDocumentOutput, BumpActionView, ResolveDocumentOutput, SessionTask};

use super::{publish_diagnostics, respond, respond_error};
use crate::state::{
    DocumentWork, ResolvedDocument, VersionLensLspState, into_lsp_code_actions, resolved_document,
};

pub(super) struct Completion {
    id: RequestId,
    output: Result<TaskOutput, String>,
}

enum TaskOutput {
    Analyzed(AnalyzeDocumentOutput),
    Resolved(ResolveDocumentOutput),
    Actions(Vec<BumpActionView>),
}

const PANICKED: &str = "VersionLens task panicked";

enum RequestKind {
    Lenses,
    Actions,
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
    runtime: Handle,
}

struct CompletedDocument {
    generation: u64,
    resolved: ResolvedDocument,
}

struct PendingApplication {
    original: RequestId,
    deadline: Instant,
}

struct Submission<F> {
    id: RequestId,
    work: DocumentWork,
    job: F,
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
    pub(super) fn new(runtime: Handle) -> Self {
        let (sender, receiver) = unbounded();
        Self {
            pending: HashMap::new(),
            completed: HashMap::new(),
            sender,
            receiver,
            applications: HashMap::new(),
            next_id: 0,
            runtime,
        }
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
        // The freshness check and the analysis read the persistent cache, so
        // they run on the runtime and not on the protocol thread.
        let session = state.session_for(uri).clone();
        let input = work.input.clone();
        let job = async move {
            let fresh = {
                let session = session.clone();
                let input = input.clone();
                tokio::task::spawn_blocking(move || session.document_is_fresh(&input))
                    .await
                    .map_err(|_| PANICKED.to_owned())?
            };
            if !fresh {
                session
                    .run_task(SessionTask::Resolve(input.clone()))
                    .await?;
            }
            tokio::task::spawn_blocking(move || session.analyze_document(input))
                .await
                .map(TaskOutput::Analyzed)
                .map_err(|_| PANICKED.to_owned())
        };
        self.submit(
            connection,
            Submission {
                id,
                work,
                job,
                kind: RequestKind::Lenses,
            },
        )
    }

    /// Answers a code action request from cached results only, so it never
    /// waits for a registry.
    pub(super) fn actions(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        id: RequestId,
        uri: &str,
        range: lsp_types::Range,
    ) -> Result<()> {
        let Some(work) = state.document_work(uri) else {
            return respond(connection, id, json!([]));
        };
        let range = versionlens_model::Range {
            start: versionlens_model::Position {
                line: range.start.line,
                character: range.start.character,
            },
            end: versionlens_model::Position {
                line: range.end.line,
                character: range.end.character,
            },
        };
        let session = state.session_for(uri).clone();
        let input = work.input.clone();
        let job = async move {
            tokio::task::spawn_blocking(move || session.bump_actions(&input, range))
                .await
                .map(TaskOutput::Actions)
                .map_err(|_| PANICKED.to_owned())
        };
        self.submit(
            connection,
            Submission {
                id,
                work,
                job,
                kind: RequestKind::Actions,
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
        let session = state.session_for(&args.uri).clone();
        let job = async move { session.run_task(task).await.map(TaskOutput::Resolved) };
        self.submit(
            connection,
            Submission {
                id,
                work,
                job,
                kind: RequestKind::Update,
            },
        )
    }

    fn submit<F>(&mut self, connection: &Connection, submission: Submission<F>) -> Result<()>
    where
        F: Future<Output = Result<TaskOutput, String>> + Send + 'static,
    {
        let Submission {
            id,
            work,
            job,
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
        let run = self.runtime.spawn(job);
        let cancellation = run.abort_handle();
        self.runtime.spawn(async move {
            let output = match run.await {
                Ok(output) => output,
                Err(error) if error.is_panic() => Err(PANICKED.to_owned()),
                Err(_) => return,
            };
            let _ = sender.send(Completion {
                id: completion_id,
                output,
            });
        });
        self.pending.insert(
            id,
            Pending {
                work,
                cancellation,
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
        match (pending.kind, output) {
            (RequestKind::Lenses, TaskOutput::Analyzed(analysis)) => {
                let resolved =
                    resolved_document(analysis, &pending.work.uri, pending.work.generation);
                let result =
                    respond_with_document(connection, completion.id, &pending.work, &resolved);
                self.completed.insert(
                    pending.work.uri,
                    CompletedDocument {
                        generation: pending.work.generation,
                        resolved,
                    },
                );
                result
            }
            (RequestKind::Actions, TaskOutput::Actions(actions)) => {
                let actions = into_lsp_code_actions(
                    actions,
                    &pending.work.uri.parse()?,
                    pending.work.version,
                    state.client.document_changes,
                );
                respond(connection, completion.id, serde_json::to_value(actions)?)
            }
            (RequestKind::Update, TaskOutput::Resolved(output)) => {
                self.apply(state, connection, completion.id, output)
            }
            _ => respond_error(
                connection,
                completion.id,
                ErrorCode::InternalError,
                "unexpected task output".to_owned(),
            ),
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

    pub(super) fn next_expiry(&self) -> Option<Instant> {
        self.applications
            .values()
            .map(|application| application.deadline)
            .min()
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
