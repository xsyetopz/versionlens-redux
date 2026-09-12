use std::panic;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use versionlens_model::DocumentInput;

use super::{ApplyCommandRequest, VersionLensSession};
use crate::ResolveDocumentOutput;
use crate::concurrency::{self, Priority};

pub enum SessionTask {
    Resolve(DocumentInput),
    Background(DocumentInput),
    Command {
        document: DocumentInput,
        command: Option<String>,
        dependency: Option<String>,
        selected_version: Option<String>,
    },
}

pub struct TaskCancellation(Arc<AtomicBool>);

impl TaskCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

impl Drop for TaskCancellation {
    fn drop(&mut self) {
        self.cancel();
    }
}

type Completion = Box<dyn FnOnce(Result<ResolveDocumentOutput, String>) + Send>;

struct QueuedTask {
    session: VersionLensSession,
    task: SessionTask,
    complete: Completion,
}

impl QueuedTask {
    fn run(self) {
        let Self {
            session,
            task,
            complete,
        } = self;
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            if !session.operation_context().is_current() {
                return Err("checking cancelled".to_owned());
            }
            let output = match task {
                SessionTask::Resolve(document) | SessionTask::Background(document) => {
                    session.resolve_document(document)
                }
                SessionTask::Command {
                    document,
                    command,
                    dependency,
                    selected_version,
                } => session.apply_command_with_selected_version(ApplyCommandRequest {
                    input: document,
                    command: command.as_deref(),
                    dependency_name: dependency.as_deref(),
                    selected_version: selected_version.as_deref(),
                    responses: &[],
                }),
            };
            if !session.operation_context().is_current() {
                return Err("checking cancelled".to_owned());
            }
            Ok(output)
        }))
        .unwrap_or_else(|_| Err("resolution worker failed".to_owned()));
        complete(result);
    }
}

impl VersionLensSession {
    pub fn submit_task(
        &self,
        task: SessionTask,
        complete: impl FnOnce(Result<ResolveDocumentOutput, String>) + Send + 'static,
    ) -> Result<TaskCancellation, &'static str> {
        self.synchronize_persistent_cache();
        let cancellation = Arc::new(AtomicBool::new(false));
        let mut session = self.clone();
        session.storage_state.task_epoch =
            Some(self.storage_state.cache_epoch.load(Ordering::Acquire));
        session.storage_state.task_cancellation = Some(Arc::clone(&cancellation));
        let priority = if matches!(task, SessionTask::Background(_)) {
            Priority::Background
        } else {
            Priority::Interactive
        };
        session.storage_state.task_priority = priority;
        let task = QueuedTask {
            session,
            task,
            complete: Box::new(complete),
        };
        concurrency::schedule_coordinator(priority, move || task.run())?;
        Ok(TaskCancellation(cancellation))
    }
}
