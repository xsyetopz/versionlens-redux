use versionlens_parsers::Dependency;
use versionlens_suggestions::Suggestion;
use versionlens_versions::ProjectVersionBump;

use crate::VersionLensSession;
use crate::model::RegistryResponseInput;
use crate::registry::RegistryContext;

mod dependency;
mod latest;
mod parallel;

impl VersionLensSession {
    pub(crate) fn resolve_dependencies(
        &self,
        dependencies: Vec<Dependency>,
        document_uri: &str,
        responses: &[RegistryResponseInput],
        project_bump: Option<ProjectVersionBump>,
        context: &RegistryContext,
    ) -> Vec<Suggestion> {
        parallel::resolve_dependencies(
            self,
            dependencies,
            document_uri,
            responses,
            project_bump,
            context,
        )
    }
}

#[cfg(test)]
mod tests;
