use std::panic;
use std::sync::{Arc, mpsc};
use versionlens_suggestions::{Suggestion, error};

use super::ResolutionRequest;
use super::dependency::ResolveDependencyInput;
use crate::{VersionLensSession, concurrency, workspace};

const WORKER_PANIC_MESSAGE: &str = "dependency resolution worker panicked";
const OPERATION_TIMEOUT_MESSAGE: &str = "dependency resolution timed out";

pub(super) fn resolve_dependencies(
    session: &VersionLensSession,
    request: ResolutionRequest<'_>,
) -> Vec<Suggestion> {
    let workspace = Arc::new(session.workspace_graph(request.input));
    let workspace_documents = session.workspace_documents();
    let workspace_root = Arc::new(
        workspace::document_path(request.input)
            .and_then(|_| workspace::workspace_root(request.input)),
    );
    let context = Arc::new(request.context.clone());
    let session = Arc::new(session.clone());
    let responses = Arc::new(request.responses.to_vec());
    let document_uri = Arc::new(request.document_uri.to_owned());
    let (sender, receiver) = mpsc::channel();
    let count = request.dependencies.len();
    let mut results = vec![None; count];
    for (index, dependency) in request.dependencies.into_iter().enumerate() {
        let workspace = Arc::clone(&workspace);
        let workspace_documents = Arc::clone(&workspace_documents);
        let workspace_root = Arc::clone(&workspace_root);
        let context = Arc::clone(&context);
        let session = Arc::clone(&session);
        let responses = Arc::clone(&responses);
        let document_uri = Arc::clone(&document_uri);
        let operation = request.operation.clone();
        let project_bump = request.project_bump;
        let sender = sender.clone();
        concurrency::schedule(session.storage_state.task_priority, move || {
            let operation = operation.for_execution();
            let failure = dependency.clone();
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                if operation.is_expired() {
                    return Some(error(dependency, OPERATION_TIMEOUT_MESSAGE.to_owned()));
                }
                session.resolve_dependency_with_responses(ResolveDependencyInput {
                    dependency,
                    workspace: &workspace,
                    workspace_documents: &workspace_documents,
                    workspace_root: workspace_root.as_deref(),
                    document_uri: Some(&document_uri),
                    responses: &responses,
                    project_bump,
                    context: &context,
                    operation: &operation,
                })
            }))
            .unwrap_or_else(|_| Some(error(failure, WORKER_PANIC_MESSAGE.to_owned())));
            let _ = sender.send((index, result));
        });
    }
    drop(sender);
    for (index, result) in receiver {
        results[index] = result;
    }
    results.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests;
