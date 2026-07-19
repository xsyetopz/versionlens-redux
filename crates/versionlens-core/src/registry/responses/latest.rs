use versionlens_model::Dependency;
use versionlens_providers::{LatestVersionRequest, latest_version_from_response_for_request};

use crate::VersionLensSession;
use crate::contract::RegistryResponseInput;
use crate::registry::registry_response_matches;

impl VersionLensSession {
    pub(crate) fn latest_from_responses(
        &self,
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
    ) -> Option<String> {
        responses
            .iter()
            .filter(|response| registry_response_matches(response, dependency))
            .find_map(|response| self.latest_from_response(dependency, response))
    }

    fn latest_from_response(
        &self,
        dependency: &Dependency,
        response: &RegistryResponseInput,
    ) -> Option<String> {
        latest_version_from_response_for_request(LatestVersionRequest {
            ecosystem: response.ecosystem,
            package: &response.package,
            requirement: &dependency.requirement,
            body: &response.body,
            include_prereleases: self.includes_prereleases(dependency),
            prerelease_tags: self.prerelease_tags(response.ecosystem),
        })
    }
}
