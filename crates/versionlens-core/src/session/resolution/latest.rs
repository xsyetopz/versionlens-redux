use versionlens_parsers::Dependency;

use crate::VersionLensSession;
use crate::error::FetchError;
use crate::model::RegistryResponseInput;
use crate::registry::RegistryContext;

mod cache;
mod lookup;

pub(super) struct LatestLookup {
    pub(super) latest: Option<String>,
    pub(super) builds: Vec<String>,
    pub(super) choices: Vec<versionlens_suggestions::UpdateChoice>,
    pub(super) fetch_error: Option<FetchError>,
}

impl VersionLensSession {
    pub(super) fn resolve_latest(
        &self,
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
        has_registry_response: bool,
        context: &RegistryContext,
    ) -> LatestLookup {
        if self.uses_shared_latest_cache(dependency, context) {
            return self.resolve_cacheable_latest(
                dependency,
                responses,
                has_registry_response,
                context,
            );
        }

        self.resolve_uncached_latest(dependency, responses, has_registry_response, context)
    }
}
