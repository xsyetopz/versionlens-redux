use std::thread::scope;
use versionlens_suggestions::Suggestion;

use super::ResolutionRequest;
use super::dependency::ResolveDependencyInput;
use crate::VersionLensSession;
use crate::concurrency::dependency_chunks;

type ResolvedSuggestions = Vec<Suggestion>;

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
    } = request;

    dependencies
        .into_iter()
        .filter_map(|dependency| {
            session.resolve_dependency_with_responses(ResolveDependencyInput {
                dependency,
                document_uri: Some(document_uri),
                responses,
                project_bump,
                context,
            })
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
    } = request;
    let chunks = dependency_chunks(dependencies, worker_count);
    scope(|scope| {
        let mut handles = vec![];
        for chunk in chunks {
            handles.push(scope.spawn(move || {
                chunk
                    .into_iter()
                    .filter_map(|dependency| {
                        session.resolve_dependency_with_responses(ResolveDependencyInput {
                            dependency,
                            document_uri: Some(document_uri),
                            responses,
                            project_bump,
                            context,
                        })
                    })
                    .collect::<Vec<_>>()
            }));
        }

        handles
            .into_iter()
            .filter_map(|handle| handle.join().ok())
            .flatten()
            .collect()
    })
}

#[cfg(test)]
mod tests;
