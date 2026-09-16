use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use versionlens_model::DocumentInput;

use super::{ApplyCommandRequest, VersionLensSession};
use crate::ResolveDocumentOutput;
use crate::concurrency::Priority;

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

struct OperationCancellation(Arc<AtomicBool>);

impl Drop for OperationCancellation {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

impl VersionLensSession {
    pub async fn run_task(&self, task: SessionTask) -> Result<ResolveDocumentOutput, String> {
        self.synchronize_persistent_cache();
        let cancellation = Arc::new(AtomicBool::new(false));
        let mut session = self.clone();
        session.storage_state.task_epoch =
            Some(self.storage_state.cache_epoch.load(Ordering::Acquire));
        session.storage_state.task_cancellation = Some(Arc::clone(&cancellation));
        session.storage_state.task_priority = if matches!(task, SessionTask::Background(_)) {
            Priority::Background
        } else {
            Priority::Interactive
        };
        let _cancellation = OperationCancellation(cancellation);
        if !session.operation_context().is_current() {
            return Err("checking cancelled".to_owned());
        }
        let output = match task {
            SessionTask::Resolve(document) | SessionTask::Background(document) => {
                session.resolve_document(document).await
            }
            SessionTask::Command {
                document,
                command,
                dependency,
                selected_version,
            } => {
                session
                    .apply_command_with_selected_version(ApplyCommandRequest {
                        input: document,
                        command: command.as_deref(),
                        dependency_name: dependency.as_deref(),
                        selected_version: selected_version.as_deref(),
                        responses: &[],
                    })
                    .await
            }
        };
        if session.operation_context().is_current() {
            Ok(output)
        } else {
            Err("checking cancelled".to_owned())
        }
    }
}
