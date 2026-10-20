use versionlens_model::{Dependency, DocumentInput, ProjectVersionBump};

use super::operation::OperationContext;
use crate::RegistryResponseInput;
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

#[cfg(test)]
mod tests;
