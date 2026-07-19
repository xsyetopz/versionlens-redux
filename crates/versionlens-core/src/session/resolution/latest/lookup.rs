use versionlens_model::Dependency;
use versionlens_providers::build_versions_from_response;
use versionlens_suggestions::UpdateChoice;

use crate::VersionLensSession;
use crate::contract::RegistryResponseInput;
use crate::error::FetchError;
use crate::fetch::response_update_choices;
use crate::registry::{RegistryContext, registry_response_matches};

use super::{LatestLookup, LatestResolutionRequest};

impl VersionLensSession {
    pub(in crate::session::resolution::latest) fn lookup_latest(
        &self,
        request: LatestResolutionRequest<'_>,
    ) -> Result<LatestLookup, FetchError> {
        let LatestResolutionRequest {
            dependency,
            responses,
            has_registry_response,
            context,
            operation,
        } = request;
        if has_registry_response {
            self.cache_registry_response_bodies(dependency, responses, context);
            let latest = self.latest_from_responses(dependency, responses);
            let choices = latest
                .as_deref()
                .map(|version| self.update_choices_from_responses(dependency, version, responses))
                .unwrap_or_default();
            Ok(LatestLookup {
                latest,
                builds: build_versions_from_responses(dependency, responses),
                choices,
                fetch_error: None,
            })
        } else {
            let fetched = self.fetch_latest(dependency, context, operation)?;
            Ok(LatestLookup {
                latest: fetched.latest,
                builds: fetched.builds,
                choices: fetched.choices,
                fetch_error: None,
            })
        }
    }
}

impl VersionLensSession {
    fn cache_registry_response_bodies(
        &self,
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
        context: &RegistryContext,
    ) {
        let Some(response) = matching_dependency_response(responses, dependency) else {
            return;
        };

        for endpoint in self.registry_endpoints_with_context(dependency, context) {
            let http_config =
                self.effective_http_config(&endpoint.url, dependency.ecosystem, context);
            let cache_key = self.request_cache_key(&endpoint.url, &http_config);
            self.cache_request_body(
                cache_key,
                &response.body,
                dependency.ecosystem,
                context.manifest_kind(),
            );
        }
    }
}

fn build_versions_from_responses(
    dependency: &Dependency,
    responses: &[RegistryResponseInput],
) -> Vec<String> {
    matching_dependency_response(responses, dependency)
        .map(|response| {
            build_versions_from_response(
                dependency.ecosystem,
                &response.body,
                &dependency.requirement,
            )
        })
        .unwrap_or_default()
}

fn matching_dependency_response<'a>(
    responses: &'a [RegistryResponseInput],
    dependency: &Dependency,
) -> Option<&'a RegistryResponseInput> {
    responses
        .iter()
        .find(|response| registry_response_matches(response, dependency))
}

impl VersionLensSession {
    fn update_choices_from_responses(
        &self,
        dependency: &Dependency,
        latest: &str,
        responses: &[RegistryResponseInput],
    ) -> Vec<UpdateChoice> {
        matching_dependency_response(responses, dependency)
            .map(|response| {
                response_update_choices(
                    dependency,
                    latest,
                    &response.body,
                    self.includes_prereleases(dependency),
                    self.prerelease_tags(dependency.ecosystem),
                )
            })
            .unwrap_or_default()
    }
}
