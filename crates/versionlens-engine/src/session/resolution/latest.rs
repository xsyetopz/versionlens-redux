use versionlens_model::Dependency;
use versionlens_update::UpdateChoice;

use crate::RegistryResponseInput;
use crate::error::FetchError;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;

mod cache;
mod lookup;

pub(super) struct LatestLookup {
    pub(super) latest: Option<String>,
    pub(super) builds: Vec<String>,
    pub(super) choices: Vec<UpdateChoice>,
    pub(super) fetch_error: Option<FetchError>,
    pub(super) fixed_requirement_matched: bool,
}

#[derive(Clone, Copy)]
pub(super) struct LatestResolutionRequest<'a> {
    pub(super) dependency: &'a Dependency,
    pub(super) responses: &'a [RegistryResponseInput],
    pub(super) has_registry_response: bool,
    pub(super) context: &'a RegistryContext,
    pub(super) operation: &'a OperationContext,
}
