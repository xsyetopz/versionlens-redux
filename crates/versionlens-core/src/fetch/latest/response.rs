use versionlens_model::Dependency;
use versionlens_providers::{
    LatestVersionRequest, RegistryEndpoint, latest_version_from_response_for_endpoint,
};

use crate::VersionLensSession;

impl VersionLensSession {
    pub(in crate::fetch::latest) fn latest_from_fetched_body(
        &self,
        dependency: &Dependency,
        endpoint: &RegistryEndpoint,
        body: &str,
    ) -> Option<String> {
        latest_version_from_response_for_endpoint(
            endpoint,
            LatestVersionRequest {
                ecosystem: dependency.ecosystem,
                package: dependency
                    .hosted_name
                    .as_deref()
                    .unwrap_or(&dependency.name),
                requirement: &dependency.requirement,
                body,
                include_prereleases: self.includes_prereleases(dependency),
                prerelease_tags: self.prerelease_tags(dependency.ecosystem),
            },
        )
    }
}
