use versionlens_parsers::Dependency;
use versionlens_suggestions::Suggestion;
use versionlens_versions::ProjectVersionBump;

use crate::VersionLensSession;
use crate::concurrency::dependency_chunks;
use crate::model::RegistryResponseInput;
use crate::registry::RegistryContext;

pub(super) fn resolve_dependencies(
    session: &VersionLensSession,
    dependencies: Vec<Dependency>,
    document_uri: &str,
    responses: &[RegistryResponseInput],
    project_bump: Option<ProjectVersionBump>,
    context: &RegistryContext,
) -> Vec<Suggestion> {
    let worker_count = resolve_worker_count(dependencies.len());

    if worker_count <= 1 {
        return resolve_sequential(
            session,
            dependencies,
            document_uri,
            responses,
            project_bump,
            context,
        );
    }

    resolve_parallel(
        session,
        dependencies,
        document_uri,
        responses,
        project_bump,
        worker_count,
        context,
    )
}

fn resolve_worker_count(dependency_count: usize) -> usize {
    dependency_count.min(8)
}

fn resolve_sequential(
    session: &VersionLensSession,
    dependencies: Vec<Dependency>,
    document_uri: &str,
    responses: &[RegistryResponseInput],
    project_bump: Option<ProjectVersionBump>,
    context: &RegistryContext,
) -> Vec<Suggestion> {
    dependencies
        .into_iter()
        .filter_map(|dependency| {
            session.resolve_dependency_with_responses(
                dependency,
                Some(document_uri),
                responses,
                project_bump,
                context,
            )
        })
        .collect()
}

fn resolve_parallel(
    session: &VersionLensSession,
    dependencies: Vec<Dependency>,
    document_uri: &str,
    responses: &[RegistryResponseInput],
    project_bump: Option<ProjectVersionBump>,
    worker_count: usize,
    context: &RegistryContext,
) -> Vec<Suggestion> {
    let chunks = dependency_chunks(dependencies, worker_count);
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in chunks {
            handles.push(scope.spawn(move || {
                chunk
                    .into_iter()
                    .filter_map(|dependency| {
                        session.resolve_dependency_with_responses(
                            dependency,
                            Some(document_uri),
                            responses,
                            project_bump,
                            context,
                        )
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
