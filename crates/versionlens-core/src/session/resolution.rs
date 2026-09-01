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

impl VersionLensSession {
    pub(crate) fn resolve_dependencies(&self, request: ResolutionRequest<'_>) -> Vec<Suggestion> {
        resolve_dependencies(self, request)
    }
}

#[cfg(test)]
mod tests;
