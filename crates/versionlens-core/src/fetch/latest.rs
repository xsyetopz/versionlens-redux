use semver::Version;
use versionlens_model::Dependency;
use versionlens_providers::{
    RegistryEndpoint, build_versions_from_response, release_versions_from_response_for_endpoint,
    release_versions_from_response_for_package,
};
use versionlens_suggestions::{UpdateChoice, release_update_choices_with_prereleases};

use crate::VersionLensSession;
use crate::error::FetchError;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;
use versionlens_model::Ecosystem::{Docker, Npm};
mod docker;
mod github;
use docker::docker_update_choices;
use github::{attach_github_action_replacements, github_action_versions};
pub(crate) use github::{github_action_latest, github_current_ref_is_proven};

mod body;
mod local_dotnet;
mod response;

pub(crate) struct LatestFetch {
    pub(crate) latest: Option<String>,
    pub(crate) builds: Vec<String>,
    pub(crate) choices: Vec<UpdateChoice>,
}

type UpdateChoices = Vec<UpdateChoice>;

struct ResponseUpdateRequest<'a> {
    dependency: &'a Dependency,
    endpoint: Option<&'a RegistryEndpoint>,
    latest: &'a str,
    body: &'a str,
    include_prereleases: bool,
    prerelease_tags: &'a [String],
}

impl VersionLensSession {
    pub(crate) async fn fetch_latest(
        &self,
        dependency: &Dependency,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<LatestFetch, FetchError> {
        let mut first_error = None;

        for endpoint in self.registry_endpoints_with_context(dependency, context) {
            if operation.is_expired() {
                return Err(FetchError::OperationTimeout);
            }
            match self
                .fetch_latest_from_endpoint(dependency, &endpoint, context, operation)
                .await
            {
                Ok(fetch) if fetch.latest.is_some() => return Ok(fetch),
                Ok(_) => {}
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(LatestFetch {
                latest: None,
                builds: vec![],
                choices: vec![],
            }),
        }
    }

    async fn fetch_latest_from_endpoint(
        &self,
        dependency: &Dependency,
        endpoint: &RegistryEndpoint,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<LatestFetch, FetchError> {
        let Some(body) = self
            .fetch_registry_body(dependency, &endpoint.url, context, operation)
            .await?
        else {
            return Ok(LatestFetch {
                latest: None,
                builds: vec![],
                choices: vec![],
            });
        };

        let current_ref_is_proven = if github_current_ref_is_proven(dependency, &body) {
            true
        } else {
            self.fetch_exact_github_action_ref_is_proven(dependency, endpoint, context, operation)
                .await?
        };
        let latest = current_ref_is_proven
            .then(|| {
                github_action_latest(
                    dependency,
                    &body,
                    self.includes_prereleases(dependency),
                    self.prerelease_tags(dependency.ecosystem),
                )
                .or_else(|| self.latest_from_fetched_body(dependency, endpoint, &body))
            })
            .flatten();
        let choices = latest
            .as_deref()
            .map(|version| {
                response_update_choices_with_endpoint(ResponseUpdateRequest {
                    dependency,
                    endpoint: Some(endpoint),
                    latest: version,
                    body: &body,
                    include_prereleases: self.includes_prereleases(dependency),
                    prerelease_tags: self.prerelease_tags(dependency.ecosystem),
                })
            })
            .unwrap_or_default();
        Ok(LatestFetch {
            latest,
            builds: build_versions_from_response(
                dependency.ecosystem,
                &body,
                &dependency.requirement,
            ),
            choices,
        })
    }
}

pub(crate) fn response_update_choices(
    dependency: &Dependency,
    latest: &str,
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> UpdateChoices {
    response_update_choices_with_endpoint(ResponseUpdateRequest {
        dependency,
        endpoint: None,
        latest,
        body,
        include_prereleases,
        prerelease_tags,
    })
}

fn response_update_choices_with_endpoint(request: ResponseUpdateRequest<'_>) -> UpdateChoices {
    let ResponseUpdateRequest {
        dependency,
        endpoint,
        latest,
        body,
        include_prereleases,
        prerelease_tags,
    } = request;
    if matches!(
        dependency.canonical_reference.as_ref(),
        Some(versionlens_model::CanonicalReference::GitHubActionRef { .. })
    ) {
        return vec![];
    }
    if dependency.ecosystem == Docker {
        return docker_update_choices(&dependency.requirement, latest, body);
    }

    let versions = github_action_versions(dependency, body).unwrap_or_else(|| {
        update_choice_versions_from_response(dependency, endpoint, body, latest)
    });
    let current_tag = dependency
        .canonical_reference
        .as_ref()
        .filter(|reference| {
            matches!(
                reference,
                versionlens_model::CanonicalReference::GitHubActionCommit { .. }
            )
        })
        .and_then(|reference| github::github_action_tag(reference, body));
    let requirement = current_tag
        .as_deref()
        .and_then(versionlens_versions::version_tag_parts)
        .map_or(dependency.requirement.as_str(), |(_, version)| version);
    let mut choices = release_update_choices_with_prereleases(
        requirement,
        latest,
        &versions,
        include_prereleases,
        prerelease_tags,
    );
    if current_tag.is_some() && !choices.iter().any(|choice| choice.version == latest) {
        choices.push(UpdateChoice {
            label: "latest".to_owned(),
            version: latest.to_owned(),
            command: "update".to_owned(),
            replacement: None,
        });
    }
    attach_github_action_replacements(dependency, body, &mut choices);
    choices
}

fn update_choice_versions_from_response(
    dependency: &Dependency,
    endpoint: Option<&RegistryEndpoint>,
    body: &str,
    latest: &str,
) -> Vec<String> {
    let package = dependency
        .hosted_name
        .as_deref()
        .unwrap_or(&dependency.name);
    let versions = endpoint.map_or_else(
        || release_versions_from_response_for_package(dependency.ecosystem, package, body),
        |endpoint| {
            release_versions_from_response_for_endpoint(
                endpoint,
                dependency.ecosystem,
                package,
                body,
            )
        },
    );
    if dependency.ecosystem == Npm {
        return npm_versions_capped_to_latest(versions, latest);
    }
    versions
}

fn npm_versions_capped_to_latest(versions: Vec<String>, latest: &str) -> Vec<String> {
    let Some(latest) = stable_semver(latest) else {
        return versions;
    };

    versions
        .into_iter()
        .filter(|version| {
            let Some(parsed) = crate::parse_semver(version).ok() else {
                return true;
            };
            !parsed.pre.is_empty() || semver_precedence_lte(&parsed, &latest)
        })
        .collect()
}

fn stable_semver(version: &str) -> Option<Version> {
    let version = crate::parse_semver(version.trim()).ok()?;
    version.pre.is_empty().then_some(version)
}

fn semver_precedence_lte(version: &Version, latest: &Version) -> bool {
    (version.major, version.minor, version.patch) <= (latest.major, latest.minor, latest.patch)
}

#[cfg(test)]
mod tests;
