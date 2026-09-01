use serde_json::from_str;
use std::cmp::Ordering::Greater as OrderingGreater;

use semver::Version;
use serde_json::Value;
use versionlens_model::{CanonicalReference, Dependency};
use versionlens_providers::{
    RegistryEndpoint, build_versions_from_response, release_versions_from_response_for_endpoint,
    release_versions_from_response_for_package,
};
use versionlens_suggestions::{
    UpdateChoice, push_unique_choice, release_update_choices_with_prereleases,
};

use crate::VersionLensSession;
use crate::error::FetchError;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;
use versionlens_model::Ecosystem::{Docker, GitHub, Npm};
use versionlens_versions::{latest_version_with_prerelease_tags, version_tag_parts};

mod body;
mod local_dotnet;
mod response;

pub(crate) struct LatestFetch {
    pub(crate) latest: Option<String>,
    pub(crate) builds: Vec<String>,
    pub(crate) choices: Vec<UpdateChoice>,
}

pub(crate) fn github_current_ref_is_proven(dependency: &Dependency, body: &str) -> bool {
    if dependency.ecosystem != GitHub {
        return true;
    }

    if let Some(reference) = dependency.canonical_reference.as_ref() {
        return github_action_reference_is_proven(reference, body);
    }

    let requirement = dependency.requirement.trim();
    if requirement.is_empty()
        || requirement.bytes().any(|byte| {
            matches!(
                byte,
                b' ' | b'^' | b'~' | b'<' | b'>' | b'=' | b'*' | b'|' | b','
            )
        })
    {
        return false;
    }
    let requirement = requirement.trim_start_matches(['v', 'V']);
    let requested = requirement.split('.').collect::<Vec<_>>();
    let package = dependency
        .hosted_name
        .as_deref()
        .unwrap_or(&dependency.name);
    release_versions_from_response_for_package(GitHub, package, body)
        .iter()
        .any(|release| {
            if release == requirement {
                return true;
            }
            if requested.len() > 2 || requested.iter().any(|part| part.is_empty()) {
                return false;
            }
            let release_parts = release.split('.').collect::<Vec<_>>();
            requested
                .iter()
                .enumerate()
                .all(|(index, part)| release_parts.get(index) == Some(part))
        })
}

pub(crate) fn github_action_latest(
    dependency: &Dependency,
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let reference = dependency.canonical_reference.as_ref()?;
    let current_tag = github_action_tag(reference);
    let (prefix, _) = version_tag_parts(current_tag)?;
    let tags = github_tags(body)
        .into_iter()
        .filter(|tag| tag.prefix == prefix)
        .collect::<Vec<_>>();
    latest_version_with_prerelease_tags(
        tags.iter().map(|tag| tag.version.as_str()),
        include_prereleases,
        prerelease_tags,
    )
}

#[derive(Debug)]
struct GithubTag {
    raw: String,
    prefix: String,
    version: String,
    commit: Option<String>,
}

fn github_tags(body: &str) -> Vec<GithubTag> {
    let Ok(Value::Array(entries)) = from_str::<Value>(body) else {
        return vec![];
    };
    entries
        .into_iter()
        .filter_map(|entry| {
            let raw = entry
                .as_str()
                .or_else(|| entry.get("name").and_then(Value::as_str))?;
            let (prefix, version) = version_tag_parts(raw)?;
            let commit = entry
                .get("commit")
                .and_then(|commit| commit.get("sha"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            Some(GithubTag {
                raw: raw.to_owned(),
                prefix: prefix.to_owned(),
                version: version.to_owned(),
                commit,
            })
        })
        .collect()
}

fn github_action_tag(reference: &CanonicalReference) -> &str {
    match reference {
        CanonicalReference::GitHubActionTag { tag }
        | CanonicalReference::GitHubActionSha { tag, .. } => tag,
    }
}

fn github_action_reference_is_proven(reference: &CanonicalReference, body: &str) -> bool {
    let tags = github_tags(body);
    match reference {
        CanonicalReference::GitHubActionSha { commit, tag, .. } => tags.into_iter().any(|entry| {
            entry.raw == *tag
                && entry.commit.as_deref().is_some_and(|resolved| {
                    resolved
                        .get(..commit.len())
                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(commit))
                })
        }),
        CanonicalReference::GitHubActionTag { tag } => {
            let Some((prefix, version)) = version_tag_parts(tag) else {
                return false;
            };
            let requested = version.split('.').collect::<Vec<_>>();
            tags.into_iter().any(|entry| {
                if entry.prefix != prefix {
                    return false;
                }
                if entry.raw == *tag {
                    return true;
                }
                if requested.len() > 2 || requested.iter().any(|part| part.is_empty()) {
                    return false;
                }
                let release = entry.version.split('.').collect::<Vec<_>>();
                requested
                    .iter()
                    .enumerate()
                    .all(|(index, part)| release.get(index) == Some(part))
            })
        }
    }
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

        let latest = github_current_ref_is_proven(dependency, &body)
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
    if dependency.ecosystem == Docker {
        return docker_update_choices(&dependency.requirement, latest, body);
    }

    let versions = github_action_versions(dependency, body).unwrap_or_else(|| {
        update_choice_versions_from_response(dependency, endpoint, body, latest)
    });
    let mut choices = release_update_choices_with_prereleases(
        &dependency.requirement,
        latest,
        &versions,
        include_prereleases,
        prerelease_tags,
    );
    attach_github_action_replacements(dependency, body, &mut choices);
    choices
}

fn github_action_versions(dependency: &Dependency, body: &str) -> Option<Vec<String>> {
    let reference = dependency.canonical_reference.as_ref()?;
    let (prefix, _) = version_tag_parts(github_action_tag(reference))?;
    Some(
        github_tags(body)
            .into_iter()
            .filter(|tag| tag.prefix == prefix)
            .map(|tag| tag.version)
            .collect(),
    )
}

fn attach_github_action_replacements(
    dependency: &Dependency,
    body: &str,
    choices: &mut Vec<UpdateChoice>,
) {
    let Some(CanonicalReference::GitHubActionSha { separator, .. }) =
        dependency.canonical_reference.as_ref()
    else {
        return;
    };
    let Some((prefix, _)) = dependency
        .canonical_reference
        .as_ref()
        .and_then(|reference| version_tag_parts(github_action_tag(reference)))
    else {
        choices.clear();
        return;
    };
    let tags = github_tags(body);
    choices.retain_mut(|choice| {
        let mut matches = tags.iter().filter(|tag| {
            tag.prefix == prefix
                && version_tag_parts(&choice.version)
                    .is_some_and(|(_, version)| version == tag.version)
                && tag.commit.is_some()
        });
        let Some(tag) = matches.next() else {
            return false;
        };
        if matches.next().is_some() {
            return false;
        }
        let Some(commit) = tag.commit.as_deref() else {
            return false;
        };
        choice.replacement = Some(format!("{commit}{separator}{}", tag.raw));
        true
    });
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
            replacement: None,
        }];
    };
    let updates = docker_matching_tag_shape_updates(&current, body);
    let latest_version = updates.last().map_or_else(
        || latest.to_owned(),
        |candidate| candidate.tag.as_str().to_owned(),
    );
    let mut choices = vec![];
    push_unique_choice(&mut choices, "latest", &latest_version, "update");

    if let Some(version) = docker_next_major_update(&current.numbers, &updates) {
        push_unique_choice(&mut choices, "major", version, "updateMajor");
    }
    if let Some(version) = docker_next_minor_update(&current.numbers, &updates) {
        push_unique_choice(&mut choices, "minor", version, "updateMinor");
    }
    if let Some(version) = docker_next_patch_update(&current.numbers, &updates) {
        push_unique_choice(&mut choices, "patch", version, "updatePatch");
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
        if versionlens_versions::compare_numeric_segments(&candidate.numbers, &current.numbers)
            != OrderingGreater
        {
            continue;
        }
        updates.push(DockerTagCandidate {
            tag,
            numbers: candidate.numbers,
        });
    }

    updates.sort_by(|left, right| {
        versionlens_versions::compare_numeric_segments(&left.numbers, &right.numbers)
    });
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
        .min_by(|left, right| {
            versionlens_versions::compare_numeric_segments(&left.numbers, &right.numbers)
        })
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
        .min_by(|left, right| {
            versionlens_versions::compare_numeric_segments(&left.numbers, &right.numbers)
        })
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
        .min_by(|left, right| {
            versionlens_versions::compare_numeric_segments(&left.numbers, &right.numbers)
        })
        .map(|candidate| candidate.tag.as_str())
}

fn docker_trailing_components_are_zero(numbers: &[u64], start: usize) -> bool {
    numbers.iter().skip(start).all(|value| *value == 0)
}

fn docker_tag_shape(tag: &str) -> Option<DockerTagShape> {
    let (version, suffix) = tag
        .split_once('-')
        .map_or((tag, None), |(version, suffix)| {
            (version, (!suffix.is_empty()).then_some(suffix))
        });
    let numbers = versionlens_versions::numeric_segments(version)?;
    Some(DockerTagShape {
        numbers,
        suffix: suffix.map(|value| value.to_owned()),
    })
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
