use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender, unbounded};
use lsp_server::{Connection, ErrorCode, Message, Request, RequestId, Response};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tokio::runtime::Handle;
use tokio::task::AbortHandle;
use versionlens_engine::{
    AnalyzeDocumentOutput, BumpActionView, ResolveDocumentOutput, SessionTask, UpdateMode,
};
use versionlens_model::WorkspaceEditPlan;

use super::{publish_diagnostics, respond, respond_error};
use crate::state::{
    CHOOSE_BUILD_COMMAND, DocumentWork, MANIFEST_INFO_COMMAND, ResolvedDocument, SORT_COMMAND,
    UPDATE_ALL_COMMAND, UPDATE_COMMAND, UPDATE_WORKSPACE_COMMAND, VersionLensLspState,
    into_lsp_code_actions, resolved_document,
};

pub(super) struct Completion {
    id: RequestId,
    output: Result<TaskOutput, String>,
}

enum TaskOutput {
    Analyzed(AnalyzeDocumentOutput),
    Resolved(ResolveDocumentOutput),
    Actions(Vec<BumpActionView>),
    Workspace(WorkspaceEditPlan),
}

const PANICKED: &str = "VersionLens task panicked";

/// How many manifests a workspace update resolves at once.
const WORKSPACE_UPDATE_CONCURRENCY: usize = 8;

enum RequestKind {
    Lenses,
    Actions,
    Update,
    UpdateWorkspace,
    ManifestInfo,
}

struct Pending {
    /// `None` for work across the workspace. Its edit plan is checked
    /// against the current text before it is applied.
    work: Option<DocumentWork>,
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
    work: Option<DocumentWork>,
    job: F,
    kind: RequestKind,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentArguments {
    uri: String,
}

/// One dependency, named by its selector. With no level, every update the
/// dependency has is applied as the lens offered it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateArguments {
    uri: String,
    dependency: String,
    level: Option<Level>,
    version: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateAllArguments {
    uri: String,
    #[serde(default)]
    level: Level,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateWorkspaceArguments {
    #[serde(default)]
    level: Level,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChooseBuildArguments {
    uri: String,
    dependency: String,
    build: String,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Level {
    #[default]
    Latest,
    Major,
    Minor,
    Patch,
    Release,
    Prerelease,
}

impl Level {
    const fn mode(self) -> UpdateMode {
        match self {
            Self::Latest => UpdateMode::Update,
            Self::Major => UpdateMode::UpdateMajor,
            Self::Minor => UpdateMode::UpdateMinor,
            Self::Patch => UpdateMode::UpdatePatch,
            Self::Release => UpdateMode::UpdateRelease,
            Self::Prerelease => UpdateMode::UpdatePrerelease,
        }
    }
}

/// The edit a command asks the session for.
struct EditRequest {
    uri: String,
    command: Option<UpdateMode>,
    sort: bool,
    dependency: Option<String>,
    version: Option<String>,
}

/// Adds a manifest's edits to the workspace plan. An edit another manifest
/// already made, such as a shared Cargo workspace dependency, is kept once.
fn merge_plan(
    plan: &mut WorkspaceEditPlan,
    output: Option<Result<Result<ResolveDocumentOutput, String>, tokio::task::JoinError>>,
) {
    let Some(Ok(Ok(output))) = output else {
        return;
    };
    for document in output.edit_plan.into_iter().flat_map(|plan| plan.documents) {
        match plan
            .documents
            .iter_mut()
            .find(|planned| planned.document.uri == document.document.uri)
        {
            Some(planned) => {
                for edit in document.edits {
                    if !planned.edits.contains(&edit) {
                        planned.edits.push(edit);
                    }
                }
            }
            None => plan.documents.push(document),
        }
    }
}

/// The one named-argument object of a command. Positional lists and unknown
/// fields are rejected.
fn command_arguments<T: DeserializeOwned>(arguments: Vec<Value>) -> Option<T> {
    let [argument] = <[Value; 1]>::try_from(arguments).ok()?;
    serde_json::from_value(argument).ok()
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
            // The diagnostics went out with the first answer for this
            // generation.
            return respond(
                connection,
                id,
                serde_json::to_value(completed.resolved.code_lenses.as_slice())?,
            );
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
                work: Some(work),
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
                work: Some(work),
                job,
                kind: RequestKind::Actions,
            },
        )
    }

    /// Runs a document command. Edit commands work on the current document,
    /// so a lens clicked after an edit applies to the edited text.
    pub(super) fn command(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        id: RequestId,
        command: &str,
        arguments: Vec<Value>,
    ) -> Result<()> {
        let request = match command {
            UPDATE_COMMAND => {
                command_arguments::<UpdateArguments>(arguments).map(|args| EditRequest {
                    uri: args.uri,
                    // A version alone means that version, not every update.
                    command: args
                        .level
                        .or(args.version.as_ref().map(|_| Level::Latest))
                        .map(Level::mode),
                    sort: false,
                    dependency: Some(args.dependency),
                    version: args.version,
                })
            }
            UPDATE_ALL_COMMAND => {
                command_arguments::<UpdateAllArguments>(arguments).map(|args| EditRequest {
                    uri: args.uri,
                    command: Some(args.level.mode()),
                    sort: false,
                    dependency: None,
                    version: None,
                })
            }
            CHOOSE_BUILD_COMMAND => {
                command_arguments::<ChooseBuildArguments>(arguments).map(|args| EditRequest {
                    uri: args.uri,
                    command: Some(UpdateMode::Update),
                    sort: false,
                    dependency: Some(args.dependency),
                    version: Some(args.build),
                })
            }
            SORT_COMMAND => {
                command_arguments::<DocumentArguments>(arguments).map(|args| EditRequest {
                    uri: args.uri,
                    command: None,
                    sort: true,
                    dependency: None,
                    version: None,
                })
            }
            UPDATE_WORKSPACE_COMMAND => {
                return match command_arguments::<UpdateWorkspaceArguments>(arguments) {
                    Some(args) => self.update_workspace(state, connection, id, args.level),
                    None => invalid_arguments(connection, id, command),
                };
            }
            MANIFEST_INFO_COMMAND => {
                return match command_arguments::<DocumentArguments>(arguments) {
                    Some(args) => self.manifest_info(state, connection, id, &args.uri),
                    None => invalid_arguments(connection, id, command),
                };
            }
            _ => {
                return respond_error(
                    connection,
                    id,
                    ErrorCode::InvalidParams,
                    format!("unsupported command: {command}"),
                );
            }
        };
        match request {
            Some(request) => self.edit(state, connection, id, request),
            None => invalid_arguments(connection, id, command),
        }
    }

    fn edit(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        id: RequestId,
        request: EditRequest,
    ) -> Result<()> {
        if !state.client.workspace_edits {
            return respond_error(
                connection,
                id,
                ErrorCode::RequestFailed,
                "editor does not support versioned workspace edits".to_owned(),
            );
        }
        let Some(work) = state.document_work(&request.uri) else {
            return document_not_open(connection, id, &request.uri);
        };
        let command = if request.sort {
            Some("sort".to_owned())
        } else {
            request.command.map(|mode| mode.as_str().to_owned())
        };
        let task = SessionTask::Command {
            document: work.input.clone(),
            command,
            dependency: request.dependency,
            selected_version: request.version,
        };
        let session = state.session_for(&request.uri).clone();
        let job = async move { session.run_task(task).await.map(TaskOutput::Resolved) };
        self.submit(
            connection,
            Submission {
                id,
                work: Some(work),
                job,
                kind: RequestKind::Update,
            },
        )
    }

    /// Updates every manifest that a full scan of each settings group finds,
    /// open ones from their editor text, as one workspace edit. A manifest
    /// that fails to resolve is left out.
    fn update_workspace(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        id: RequestId,
        level: Level,
    ) -> Result<()> {
        if !state.client.workspace_edits {
            return respond_error(
                connection,
                id,
                ErrorCode::RequestFailed,
                "editor does not support versioned workspace edits".to_owned(),
            );
        }
        let groups = state
            .groups()
            .iter()
            .enumerate()
            .map(|(index, group)| {
                let options = state.workspace_discovery_options(index, true);
                (group.session.clone(), options.discovery)
            })
            .collect::<Vec<_>>();
        let command = Some(level.mode().as_str().to_owned());
        let job = async move {
            let mut plan = WorkspaceEditPlan {
                documents: Vec::new(),
            };
            let mut running = tokio::task::JoinSet::new();
            for (session, discovery) in groups {
                let scan = session.clone();
                let documents = tokio::task::spawn_blocking(move || {
                    scan.discover_workspace_documents(discovery)
                        .filter_map(Result::ok)
                        .collect::<Vec<_>>()
                })
                .await
                .map_err(|_| PANICKED.to_owned())?;
                for document in documents {
                    if running.len() >= WORKSPACE_UPDATE_CONCURRENCY {
                        merge_plan(&mut plan, running.join_next().await);
                    }
                    let session = session.clone();
                    let task = SessionTask::Command {
                        document,
                        command: command.clone(),
                        dependency: None,
                        selected_version: None,
                    };
                    running.spawn(async move { session.run_task(task).await });
                }
            }
            while let Some(output) = running.join_next().await {
                merge_plan(&mut plan, Some(output));
            }
            Ok(TaskOutput::Workspace(plan))
        };
        self.submit(
            connection,
            Submission {
                id,
                work: None,
                job,
                kind: RequestKind::UpdateWorkspace,
            },
        )
    }

    fn manifest_info(
        &mut self,
        state: &VersionLensLspState,
        connection: &Connection,
        id: RequestId,
        uri: &str,
    ) -> Result<()> {
        let Some(work) = state.document_work(uri) else {
            return document_not_open(connection, id, uri);
        };
        let session = state.session_for(uri).clone();
        let input = work.input.clone();
        let job = async move {
            tokio::task::spawn_blocking(move || session.analyze_document(input))
                .await
                .map(TaskOutput::Analyzed)
                .map_err(|_| PANICKED.to_owned())
        };
        self.submit(
            connection,
            Submission {
                id,
                work: Some(work),
                job,
                kind: RequestKind::ManifestInfo,
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
            drop(sender.send(Completion {
                id: completion_id,
                output,
            }));
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
        if pending
            .work
            .as_ref()
            .is_some_and(|work| !state.work_is_current(work))
        {
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
        match (pending.kind, output, pending.work) {
            (RequestKind::Lenses, TaskOutput::Analyzed(analysis), Some(work)) => {
                let resolved = resolved_document(analysis, &work.uri);
                let result = respond_with_document(connection, completion.id, &work, &resolved);
                self.completed.insert(
                    work.uri,
                    CompletedDocument {
                        generation: work.generation,
                        resolved,
                    },
                );
                result
            }
            (RequestKind::Actions, TaskOutput::Actions(actions), Some(work)) => {
                let actions = into_lsp_code_actions(
                    actions,
                    &work.uri.parse()?,
                    work.version,
                    state.client.document_changes,
                );
                respond(connection, completion.id, serde_json::to_value(actions)?)
            }
            (RequestKind::Update, TaskOutput::Resolved(output), _) => {
                self.apply(state, connection, completion.id, output.edit_plan)
            }
            (RequestKind::UpdateWorkspace, TaskOutput::Workspace(plan), _) => {
                self.apply(state, connection, completion.id, Some(plan))
            }
            (RequestKind::ManifestInfo, TaskOutput::Analyzed(analysis), _) => respond(
                connection,
                completion.id,
                json!({
                    "supported": analysis.is_supported_manifest,
                    "provider": analysis.active_provider_name,
                    "canSortDependencies": analysis.can_sort_dependencies,
                    "installTaskConfigKey": analysis.install_task_config_key,
                    "dependencyCount": analysis.dependencies.len(),
                    "dependencySignature": analysis.dependency_signature,
                }),
            ),
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
        plan: Option<WorkspaceEditPlan>,
    ) -> Result<()> {
        let Some(plan) = plan.filter(|plan| {
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
        now: Instant,
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
            .filter(|(_, pending)| {
                pending
                    .work
                    .as_ref()
                    .is_some_and(|work| !state.work_is_current(work))
            })
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

fn invalid_arguments(connection: &Connection, id: RequestId, command: &str) -> Result<()> {
    respond_error(
        connection,
        id,
        ErrorCode::InvalidParams,
        format!("{command} takes one object of named arguments"),
    )
}

fn document_not_open(connection: &Connection, id: RequestId, uri: &str) -> Result<()> {
    respond_error(
        connection,
        id,
        ErrorCode::InvalidParams,
        format!("document is not open: {uri}"),
    )
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
mod tests {
    use std::sync::LazyLock;
    use std::time::{Duration, Instant};

    use super::*;

    fn test_runtime() -> Handle {
        static RUNTIME: LazyLock<tokio::runtime::Runtime> =
            LazyLock::new(|| tokio::runtime::Runtime::new().expect("test runtime"));
        RUNTIME.handle().clone()
    }

    fn open_test_document(state: &mut VersionLensLspState) -> DocumentWork {
        let uri = "file:///workspace/Cargo.toml";
        state.open_document(crate::state::VersionLensTextDocument {
            uri: uri.to_owned(),
            language_id: "toml".to_owned(),
            text: "[dependencies]\nserde = \"1\"\n".to_owned(),
            workspace_root: None,
        });
        state.document_work(uri).unwrap()
    }

    fn cache_test_document(queue: &mut WorkQueue, work: &DocumentWork) {
        queue.completed.insert(
            work.uri.clone(),
            CompletedDocument {
                generation: work.generation,
                resolved: ResolvedDocument {
                    code_lenses: Vec::new(),
                    diagnostics: Vec::new(),
                },
            },
        );
    }

    fn cached_test_document() -> (VersionLensLspState, DocumentWork, WorkQueue) {
        let mut state = VersionLensLspState::standard();
        let work = open_test_document(&mut state);
        let mut queue = WorkQueue::new(test_runtime());
        cache_test_document(&mut queue, &work);
        (state, work, queue)
    }

    #[test]
    fn completed_lenses_are_reused_without_submitting_session_work() {
        let (server, client) = Connection::memory();
        let (state, work, mut queue) = cached_test_document();

        queue
            .lenses(&state, &server, RequestId::from(7), work.uri.as_str())
            .expect("test setup");

        assert!(queue.pending.is_empty());
        let Message::Response(response) = client
            .receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("test setup")
        else {
            panic!("expected cached code-lens response");
        };
        assert_eq!(response.response_result.unwrap(), json!([]));
        assert!(client.receiver.try_recv().is_err());
    }

    #[test]
    fn generation_changes_invalidate_completed_lenses() {
        let (server, _client) = Connection::memory();
        let (mut state, _work, mut queue) = cached_test_document();

        state.invalidate_workspace();
        queue.invalidate(&state, &server).expect("test setup");

        assert!(queue.completed.is_empty());
    }

    #[test]
    fn unanswered_workspace_edits_finish_once_at_the_deadline_or_cancellation() {
        for cancelled in [false, true] {
            let (server, client) = Connection::memory();
            let mut queue = WorkQueue::new(test_runtime());
            let now = Instant::now();
            let original = RequestId::from(1);
            let application = RequestId::from("versionlens/apply/1".to_owned());
            queue.applications.insert(
                application.clone(),
                PendingApplication {
                    original: original.clone(),
                    deadline: now + Duration::from_secs(30),
                },
            );
            queue.expire_applications(&server, now).expect("test setup");
            assert!(client.receiver.try_recv().is_err());
            if cancelled {
                queue.cancel(&server, &original).expect("test setup");
            } else {
                queue
                    .expire_applications(&server, now + Duration::from_secs(30))
                    .expect("test setup");
            }
            let Message::Notification(cancellation) = client
                .receiver
                .recv_timeout(Duration::from_secs(1))
                .expect("test setup")
            else {
                panic!("expected cancellation of the editor request");
            };
            assert_eq!(cancellation.method, "$/cancelRequest");
            assert_eq!(cancellation.params, json!({"id":application}));
            let Message::Response(response) = client
                .receiver
                .recv_timeout(Duration::from_secs(1))
                .expect("test setup")
            else {
                panic!("expected completion of the original command");
            };
            assert_eq!(response.id, original);
            assert_eq!(
                response.response_result.unwrap_err().code,
                if cancelled {
                    ErrorCode::RequestCanceled
                } else {
                    ErrorCode::RequestFailed
                } as i32
            );
            queue
                .applied(
                    &server,
                    Response::new_ok(application, json!({"applied":true})),
                )
                .expect("test setup");
            queue
                .expire_applications(&server, now + Duration::from_secs(60))
                .expect("test setup");
            assert!(queue.applications.is_empty());
            assert!(client.receiver.try_recv().is_err());
        }
    }

    /// A lens request must not wait on the persistent cache. With `generation.json`
    /// gone, reading the cache epoch takes `cache.lock`, which another process can hold.
    #[test]
    fn lens_requests_do_not_wait_for_the_persistent_cache_lock() {
        let directory =
            versionlens_test_support::temporary_directory("lens-cache-lock").expect("test setup");
        let mut state = VersionLensLspState::standard();
        state.use_persistent_cache(&directory).expect("test setup");
        let uri = "file:///workspace/Cargo.toml";
        state.open_document(crate::state::VersionLensTextDocument {
            uri: uri.to_owned(),
            language_id: "toml".to_owned(),
            text: "[package]\nname = \"example\"\n".to_owned(),
            workspace_root: None,
        });
        drop(std::fs::remove_file(directory.join("generation.json")));
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("cache.lock"))
            .expect("test setup");
        lock.lock().expect("test setup");
        let release = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            drop(lock);
        });
        let (server, client) = Connection::memory();
        let mut queue = WorkQueue::new(test_runtime());

        let started = Instant::now();
        queue
            .lenses(&state, &server, RequestId::from(1), uri)
            .expect("test setup");
        let elapsed = started.elapsed();
        release
            .join()
            .map_err(|_| anyhow::anyhow!("lock holder panicked"))
            .expect("test setup");
        assert!(
            elapsed < Duration::from_millis(500),
            "lens request blocked the protocol thread for {elapsed:?}"
        );

        let completion = queue
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("test setup");
        queue
            .complete(&state, &server, completion)
            .expect("test setup");
        let Message::Response(response) = client
            .receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("test setup")
        else {
            panic!("expected the code lens response first");
        };
        assert_eq!(response.id, RequestId::from(1));
        assert!(response.response_result.is_ok_and(|value| value.is_array()));
    }

    /// Opening and editing a document run on the protocol thread, so they must
    /// not read the persistent cache, whose lock another process can hold.
    #[test]
    fn opening_and_changing_documents_do_not_wait_for_the_persistent_cache_lock() {
        let directory =
            versionlens_test_support::temporary_directory("open-cache-lock").expect("test setup");
        let mut state = VersionLensLspState::standard();
        state.use_persistent_cache(&directory).expect("test setup");
        drop(std::fs::remove_file(directory.join("generation.json")));
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("cache.lock"))
            .expect("test setup");
        lock.lock().expect("test setup");
        let release = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            drop(lock);
        });
        let uri = "file:///workspace/Cargo.toml";

        let started = Instant::now();
        state.open_document(crate::state::VersionLensTextDocument {
            uri: uri.to_owned(),
            language_id: "toml".to_owned(),
            text: "[dependencies]\nserde = \"1\"\n".to_owned(),
            workspace_root: None,
        });
        state.change_document(uri, "[dependencies]\nserde = \"1.0\"\n".to_owned());
        let elapsed = started.elapsed();
        release
            .join()
            .map_err(|_| anyhow::anyhow!("lock holder panicked"))
            .expect("test setup");
        assert!(
            elapsed < Duration::from_millis(500),
            "document notifications blocked the protocol thread for {elapsed:?}"
        );
    }
}
