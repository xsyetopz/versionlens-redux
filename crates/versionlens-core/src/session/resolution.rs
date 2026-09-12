use self::parallel::resolve_dependencies;
use versionlens_model::{Dependency, DocumentInput};
use versionlens_suggestions::Suggestion;
use versionlens_versions::ProjectVersionBump;

use super::operation::OperationContext;
use crate::RegistryResponseInput;
use crate::VersionLensSession;
use crate::registry::RegistryContext;

pub(crate) struct ResolutionRequest<'a> {
    pub(super) input: &'a DocumentInput,
    pub(super) dependencies: Vec<Dependency>,
    pub(super) document_uri: &'a str,
    pub(super) responses: &'a [RegistryResponseInput],
    pub(super) project_bump: Option<ProjectVersionBump>,
    pub(super) context: &'a RegistryContext,
    pub(super) operation: &'a OperationContext,
}

mod dependency;
mod latest;
mod parallel;
mod runtime;

impl VersionLensSession {
    pub(crate) fn resolve_dependencies(&self, request: ResolutionRequest<'_>) -> Vec<Suggestion> {
        let responses = request.responses;
        let operation = request.operation;
        let manifest_kind = request.context.manifest_kind();
        let suggestions = resolve_dependencies(self, request);
        if self.config.show_vulnerabilities {
            for suggestion in &suggestions {
                self.cache_vulnerabilities(
                    &suggestion.dependency,
                    responses,
                    manifest_kind,
                    operation,
                );
            }
        }
        suggestions
    }
}

#[cfg(test)]
mod tests;
