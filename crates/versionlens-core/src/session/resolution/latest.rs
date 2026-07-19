use versionlens_model::Dependency;
use versionlens_suggestions::UpdateChoice;

use crate::VersionLensSession;
use crate::contract::RegistryResponseInput;
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
}

#[derive(Clone, Copy)]
pub(super) struct LatestResolutionRequest<'a> {
    pub(super) dependency: &'a Dependency,
    pub(super) responses: &'a [RegistryResponseInput],
    pub(super) has_registry_response: bool,
    pub(super) context: &'a RegistryContext,
    pub(super) operation: &'a OperationContext,
}

impl VersionLensSession {
    pub(super) fn resolve_latest(&self, request: LatestResolutionRequest<'_>) -> LatestLookup {
        if self.uses_shared_latest_cache(request.dependency, request.context) {
            return self.resolve_cacheable_latest(request);
        }

        self.resolve_uncached_latest(request)
    }
}
