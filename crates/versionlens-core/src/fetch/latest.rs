use serde_json::from_str;
use std::cmp::Ordering;
use std::cmp::Ordering::{Equal as OrderingEqual, Greater as OrderingGreater};

use semver::Version;
use serde_json::Value;
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
    pub(crate) fn fetch_latest(
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
            match self.fetch_latest_from_endpoint(dependency, &endpoint, context, operation) {
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

    fn fetch_latest_from_endpoint(
        &self,
        dependency: &Dependency,
        endpoint: &RegistryEndpoint,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<LatestFetch, FetchError> {
        let Some(body) = self.fetch_registry_body(dependency, &endpoint.url, context, operation)?
        else {
            return Ok(LatestFetch {
                latest: None,
                builds: vec![],
                choices: vec![],
            });
        };

        let latest = self.latest_from_fetched_body(dependency, endpoint, &body);
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
    if dependency.ecosystem == Docker {
        return docker_update_choices(&dependency.requirement, latest, body);
    }

    let versions = update_choice_versions_from_response(dependency, endpoint, body, latest);
    release_update_choices_with_prereleases(
        &dependency.requirement,
        latest,
        &versions,
        include_prereleases,
        prerelease_tags,
    )
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

fn docker_update_choices(requirement: &str, latest: &str, body: &str) -> UpdateChoices {
    if latest.is_empty() || latest == requirement {
        return vec![];
    }

    let Some(current) = docker_tag_shape(requirement) else {
        return vec![UpdateChoice {
            label: "latest".to_owned(),
            version: latest.to_owned(),
            command: "update".to_owned(),
        }];
    };
    let updates = docker_matching_tag_shape_updates(&current, body);
    let latest_version = updates.last().map_or_else(
        || latest.to_owned(),
        |candidate| candidate.tag.as_str().to_owned(),
    );
    let mut choices = vec![];
    push_unique_docker_choice(&mut choices, "latest", &latest_version, "update");

    if let Some(version) = docker_next_major_update(&current.numbers, &updates) {
        push_unique_docker_choice(&mut choices, "major", version, "updateMajor");
    }
    if let Some(version) = docker_next_minor_update(&current.numbers, &updates) {
        push_unique_docker_choice(&mut choices, "minor", version, "updateMinor");
    }
    if let Some(version) = docker_next_patch_update(&current.numbers, &updates) {
        push_unique_docker_choice(&mut choices, "patch", version, "updatePatch");
    }

    choices
}

struct DockerTagShape {
    numbers: Vec<u64>,
    suffix: Option<String>,
}

struct DockerTagCandidate {
    tag: String,
    numbers: Vec<u64>,
}

fn docker_matching_tag_shape_updates(
    current: &DockerTagShape,
    body: &str,
) -> Vec<DockerTagCandidate> {
    let tags = docker_response_tag_names(body);
    let mut updates = vec![];

    for tag in tags {
        let Some(candidate) = docker_tag_shape(&tag) else {
            continue;
        };
        if candidate.suffix != current.suffix || candidate.numbers.len() != current.numbers.len() {
            continue;
        }
        if compare_docker_numbers(&candidate.numbers, &current.numbers) != OrderingGreater {
            continue;
        }
        updates.push(DockerTagCandidate {
            tag,
            numbers: candidate.numbers,
        });
    }

    updates.sort_by(|left, right| compare_docker_numbers(&left.numbers, &right.numbers));
    updates
}

fn docker_next_major_update<'a>(
    current: &[u64],
    updates: &'a [DockerTagCandidate],
) -> Option<&'a str> {
    updates
        .iter()
        .filter(|candidate| {
            candidate.numbers.first() > current.first()
                && docker_trailing_components_are_zero(&candidate.numbers, 1)
        })
        .min_by(|left, right| compare_docker_numbers(&left.numbers, &right.numbers))
        .map(|candidate| candidate.tag.as_str())
}

fn docker_next_minor_update<'a>(
    current: &[u64],
    updates: &'a [DockerTagCandidate],
) -> Option<&'a str> {
    let major = *current.first()?;
    let minor = *current.get(1)?;
    updates
        .iter()
        .filter(|candidate| {
            candidate.numbers.first() == Some(&major)
                && candidate.numbers.get(1).is_some_and(|value| *value > minor)
                && docker_trailing_components_are_zero(&candidate.numbers, 2)
        })
        .min_by(|left, right| compare_docker_numbers(&left.numbers, &right.numbers))
        .map(|candidate| candidate.tag.as_str())
}

fn docker_next_patch_update<'a>(
    current: &[u64],
    updates: &'a [DockerTagCandidate],
) -> Option<&'a str> {
    let major = *current.first()?;
    let minor = *current.get(1)?;
    let patch = *current.get(2)?;
    updates
        .iter()
        .filter(|candidate| {
            candidate.numbers.first() == Some(&major)
                && candidate.numbers.get(1) == Some(&minor)
                && candidate.numbers.get(2).is_some_and(|value| *value > patch)
                && docker_trailing_components_are_zero(&candidate.numbers, 3)
        })
        .min_by(|left, right| compare_docker_numbers(&left.numbers, &right.numbers))
        .map(|candidate| candidate.tag.as_str())
}

fn docker_trailing_components_are_zero(numbers: &[u64], start: usize) -> bool {
    numbers.iter().skip(start).all(|value| *value == 0)
}

fn push_unique_docker_choice(
    choices: &mut Vec<UpdateChoice>,
    label: &str,
    version: &str,
    command: &str,
) {
    if choices.iter().any(|choice| choice.version == version) {
        return;
    }

    choices.push(UpdateChoice {
        label: label.to_owned(),
        version: version.to_owned(),
        command: command.to_owned(),
    });
}

fn docker_tag_shape(tag: &str) -> Option<DockerTagShape> {
    let (version, suffix) = tag
        .split_once('-')
        .map_or((tag, None), |(version, suffix)| {
            (version, (!suffix.is_empty()).then_some(suffix))
        });
    let numbers = version
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (!numbers.is_empty()).then_some(DockerTagShape {
        numbers,
        suffix: suffix.map(|value| value.to_owned()),
    })
}

fn compare_docker_numbers(left: &[u64], right: &[u64]) -> Ordering {
    let len = left.len().max(right.len());
    (0..len)
        .map(|index| {
            left.get(index)
                .unwrap_or(&0)
                .cmp(right.get(index).unwrap_or(&0))
        })
        .find(|ordering| *ordering != OrderingEqual)
        .unwrap_or(OrderingEqual)
}

fn docker_response_tag_names(body: &str) -> Vec<String> {
    let Ok(value) = from_str::<Value>(body) else {
        return vec![];
    };
    let mut tags = vec![];
    tags.extend(docker_object_tag_names(
        value.get("results").unwrap_or(&value),
    ));
    tags.extend(docker_registry_v2_tag_names(&value));
    tags
}

fn docker_object_tag_names(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flat_map(|tags| tags.iter())
        .filter_map(docker_object_tag_name)
        .map(|value| value.to_owned())
        .collect()
}

fn docker_object_tag_name(entry: &Value) -> Option<&str> {
    let status = entry.get("tag_status").and_then(|value| value.as_str());
    if status.is_some_and(|status| status != "active") {
        return None;
    }
    if status.is_some()
        && entry
            .get("digest")
            .and_then(|value| value.as_str())
            .is_none_or(str::is_empty)
    {
        return None;
    }
    entry.get("name")?.as_str()
}

fn docker_registry_v2_tag_names(value: &Value) -> Vec<String> {
    value
        .get("tags")
        .and_then(|value| value.as_array())
        .into_iter()
        .flat_map(|tags| tags.iter())
        .filter_map(|value| value.as_str())
        .map(|value| value.to_owned())
        .collect()
}

#[cfg(test)]
mod tests;
