use std::thread::{ScopedJoinHandle, scope};
use versionlens_model::Dependency;
use versionlens_suggestions::Suggestion;
use versionlens_suggestions::error;

use super::ResolutionRequest;
use super::dependency::ResolveDependencyInput;
use crate::VersionLensSession;
use crate::concurrency::dependency_chunks;

type ResolvedSuggestions = Vec<Suggestion>;
const WORKER_PANIC_MESSAGE: &str = "dependency resolution worker panicked";
const OPERATION_TIMEOUT_MESSAGE: &str = "dependency resolution timed out";

pub(super) fn resolve_dependencies(
    session: &VersionLensSession,
    request: ResolutionRequest<'_>,
) -> ResolvedSuggestions {
    let worker_count = resolve_worker_count(request.dependencies.len());

    if worker_count <= 1 {
        return resolve_sequential(session, request);
    }

    resolve_parallel(session, request, worker_count)
}

fn resolve_worker_count(dependency_count: usize) -> usize {
    dependency_count.min(8)
}

fn resolve_sequential(
    session: &VersionLensSession,
    request: ResolutionRequest<'_>,
) -> ResolvedSuggestions {
    let ResolutionRequest {
        dependencies,
        document_uri,
        responses,
        project_bump,
        context,
        operation,
    } = request;

    dependencies
        .into_iter()
        .filter_map(|dependency| {
            resolve_dependency(
                session,
                ResolveDependencyInput {
                    dependency,
                    document_uri: Some(document_uri),
                    responses,
                    project_bump,
                    context,
                    operation,
                },
            )
        })
        .collect()
}

fn resolve_parallel(
    session: &VersionLensSession,
    request: ResolutionRequest<'_>,
    worker_count: usize,
) -> ResolvedSuggestions {
    let ResolutionRequest {
        dependencies,
        document_uri,
        responses,
        project_bump,
        context,
        operation,
    } = request;
    let chunks = dependency_chunks(dependencies, worker_count);
    scope(|scope| {
        let mut handles = vec![];
        for chunk in chunks {
            let fallback_dependencies = chunk.clone();
            let handle = scope.spawn(move || {
                chunk
                    .into_iter()
                    .filter_map(|dependency| {
                        resolve_dependency(
                            session,
                            ResolveDependencyInput {
                                dependency,
                                document_uri: Some(document_uri),
                                responses,
                                project_bump,
                                context,
                                operation,
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            });
            handles.push((handle, fallback_dependencies));
        }

        handles
            .into_iter()
            .flat_map(|(handle, fallback_dependencies)| join_worker(handle, fallback_dependencies))
            .collect()
    })
}

fn resolve_dependency(
    session: &VersionLensSession,
    input: ResolveDependencyInput<'_>,
) -> Option<Suggestion> {
    if input.operation.is_expired() {
        return Some(error(
            input.dependency,
            OPERATION_TIMEOUT_MESSAGE.to_owned(),
        ));
    }

    session.resolve_dependency_with_responses(input)
}

fn join_worker(
    handle: ScopedJoinHandle<'_, ResolvedSuggestions>,
    fallback_dependencies: Vec<Dependency>,
) -> ResolvedSuggestions {
    handle.join().unwrap_or_else(|_| {
        fallback_dependencies
            .into_iter()
            .map(|dependency| error(dependency, WORKER_PANIC_MESSAGE.to_owned()))
            .collect()
    })
}

#[cfg(test)]
mod tests;
