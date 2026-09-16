use std::{iter, sync::Arc};

use versionlens_suggestions::{Suggestion, error};

use super::ResolutionRequest;
use super::dependency::ResolveDependencyInput;
use crate::{VersionLensSession, workspace};

const WORKER_PANIC_MESSAGE: &str = "dependency resolution worker panicked";
const OPERATION_TIMEOUT_MESSAGE: &str = "dependency resolution timed out";
pub(super) async fn resolve_dependencies(
    session: &VersionLensSession,
    request: ResolutionRequest<'_>,
) -> Vec<Suggestion> {
    let cache_scope = session.document_cache_scope(request.context, request.input);
    let use_cached_suggestions = request.responses.is_empty() && request.project_bump.is_none();
    let mut dependencies = Vec::with_capacity(request.dependencies.len());
    let mut suggestions = iter::repeat_with(|| None)
        .take(request.dependencies.len())
        .collect::<Vec<_>>();
    for (index, dependency) in request.dependencies.into_iter().enumerate() {
        if use_cached_suggestions
            && let Some(cached) = session.cached_resolved_suggestion(&dependency, &cache_scope)
        {
            suggestions[index] = Some(cached);
        } else {
            dependencies.push((index, dependency));
        }
    }

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
    let mut tasks = Vec::with_capacity(dependencies.len());

    for (index, dependency) in dependencies {
        let failure = dependency.clone();
        let workspace = Arc::clone(&workspace);
        let workspace_documents = Arc::clone(&workspace_documents);
        let workspace_root = Arc::clone(&workspace_root);
        let context = Arc::clone(&context);
        let session = Arc::clone(&session);
        let responses = Arc::clone(&responses);
        let document_uri = Arc::clone(&document_uri);
        let operation = request.operation.clone();
        let project_bump = request.project_bump;
        let task = tokio::spawn(async move {
            if operation.is_expired() {
                return Some(error(dependency, OPERATION_TIMEOUT_MESSAGE.to_owned()));
            }
            session
                .resolve_dependency_with_responses(ResolveDependencyInput {
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
                .await
        });
        tasks.push((index, task, failure));
    }

    for (index, task, failure) in tasks {
        suggestions[index] = match task.await {
            Ok(suggestion) => suggestion,
            Err(_) => Some(error(failure, WORKER_PANIC_MESSAGE.to_owned())),
        };
    }
    suggestions.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests;
