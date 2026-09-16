use std::collections::HashSet;

use crate::{
    VersionLensSession, error::FetchError, registry::RegistryContext,
    session::operation::OperationContext,
};
use serde_json::{Value, from_str};
use versionlens_model::{CanonicalReference, Dependency, Ecosystem::GitHub};
use versionlens_providers::{
    RegistryEndpoint, github_tag_ref_url, release_versions_from_response_for_package,
};
use versionlens_suggestions::UpdateChoice;
use versionlens_versions::{latest_version_with_prerelease_tags, version_tag_parts};

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
    if let CanonicalReference::GitHubActionRef { reference } = reference {
        return Some(reference.clone());
    }
    let current_tag = github_action_tag(reference, body)?;
    let (prefix, _) = version_tag_parts(&current_tag)?;
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
                .map(str::to_ascii_lowercase);
            Some(GithubTag {
                raw: raw.to_owned(),
                prefix: prefix.to_owned(),
                version: version.to_owned(),
                commit,
            })
        })
        .collect()
}

pub(super) fn github_action_tag(reference: &CanonicalReference, body: &str) -> Option<String> {
    match reference {
        CanonicalReference::GitHubActionTag { tag }
        | CanonicalReference::GitHubActionSha { tag, .. } => Some(tag.clone()),
        CanonicalReference::GitHubActionCommit { commit } => {
            let tags = github_tags(body);
            let matching = tags
                .iter()
                .filter(|tag| {
                    tag.commit
                        .as_deref()
                        .is_some_and(|sha| sha_prefix_matches(commit, sha))
                })
                .collect::<Vec<_>>();
            let first = matching.first()?;
            if matching
                .iter()
                .any(|tag| tag.prefix != first.prefix || tag.commit != first.commit)
            {
                return None;
            }
            let latest = versionlens_versions::latest_version(
                matching.iter().map(|tag| tag.version.as_str()),
                true,
            )?;
            matching
                .into_iter()
                .find(|tag| tag.version == latest)
                .map(|tag| tag.raw.clone())
        }
        CanonicalReference::GitHubActionRef { .. }
        | CanonicalReference::GitHubActionLocal { .. }
        | CanonicalReference::GitHubActionExpression { .. }
        | CanonicalReference::GitHubActionInvalid { .. } => None,
    }
}

fn github_action_reference_is_proven(reference: &CanonicalReference, body: &str) -> bool {
    let tags = github_tags(body);
    match reference {
        CanonicalReference::GitHubActionCommit { .. } => {
            github_action_tag(reference, body).is_some()
        }
        CanonicalReference::GitHubActionSha { commit, tag, .. } => tags.into_iter().any(|entry| {
            entry.raw == *tag
                && entry
                    .commit
                    .as_deref()
                    .is_some_and(|resolved| sha_prefix_matches(commit, resolved))
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
        CanonicalReference::GitHubActionRef { .. }
        | CanonicalReference::GitHubActionLocal { .. }
        | CanonicalReference::GitHubActionExpression { .. }
        | CanonicalReference::GitHubActionInvalid { .. } => false,
    }
}

pub(super) fn github_action_reference_is_proven_by_exact_ref(
    reference: &CanonicalReference,
    body: &str,
) -> bool {
    let CanonicalReference::GitHubActionSha { commit, tag, .. } = reference else {
        return false;
    };
    let Ok(value) = from_str::<Value>(body) else {
        return false;
    };
    let expected_ref = format!("refs/tags/{tag}");
    let object = value.get("object");
    value.get("ref").and_then(Value::as_str) == Some(expected_ref.as_str())
        && matches!(
            object
                .and_then(|object| object.get("type"))
                .and_then(Value::as_str),
            Some("commit")
        )
        && object
            .and_then(|object| object.get("sha"))
            .and_then(Value::as_str)
            .is_some_and(|resolved| sha_prefix_matches(commit, resolved))
}

fn sha_prefix_matches(expected: &str, resolved: &str) -> bool {
    (7..=40).contains(&expected.len())
        && expected.bytes().all(|byte| byte.is_ascii_hexdigit())
        && resolved.len() == 40
        && resolved.bytes().all(|byte| byte.is_ascii_hexdigit())
        && resolved
            .get(..expected.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(expected))
}

pub(super) fn github_action_versions(dependency: &Dependency, body: &str) -> Option<Vec<String>> {
    let reference = dependency.canonical_reference.as_ref()?;
    let current_tag = github_action_tag(reference, body)?;
    let (prefix, _) = version_tag_parts(&current_tag)?;
    Some(
        github_tags(body)
            .into_iter()
            .filter(|tag| tag.prefix == prefix)
            .map(|tag| tag.version)
            .collect(),
    )
}

pub(super) fn attach_github_action_replacements(
    dependency: &Dependency,
    body: &str,
    choices: &mut Vec<UpdateChoice>,
) {
    let (reference, separator) = match dependency.canonical_reference.as_ref() {
        Some(reference @ CanonicalReference::GitHubActionSha { separator, .. }) => {
            (reference, Some(separator.as_str()))
        }
        Some(reference @ CanonicalReference::GitHubActionCommit { .. }) => (reference, None),
        _ => return,
    };
    let Some(current_tag) = github_action_tag(reference, body) else {
        choices.clear();
        return;
    };
    let Some((prefix, current_version)) = version_tag_parts(&current_tag) else {
        choices.clear();
        return;
    };
    let tags = github_tags(body);
    choices.retain_mut(|choice| {
        if matches!(reference, CanonicalReference::GitHubActionCommit { .. })
            && !matches!(
                versionlens_versions::compare_versions(&choice.version, current_version),
                Some(std::cmp::Ordering::Equal | std::cmp::Ordering::Greater)
            )
        {
            return false;
        }
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
        if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return false;
        }
        choice.replacement = Some(separator.map_or_else(
            || commit.to_owned(),
            |separator| format!("{commit}{separator}{}", tag.raw),
        ));
        true
    });
}

impl VersionLensSession {
    pub(super) async fn fetch_exact_github_action_ref_is_proven(
        &self,
        dependency: &Dependency,
        endpoint: &RegistryEndpoint,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<bool, FetchError> {
        let Some(reference) = dependency.canonical_reference.as_ref() else {
            return Ok(false);
        };
        if let CanonicalReference::GitHubActionRef { reference } = reference {
            return self
                .fetch_floating_github_action_ref(
                    endpoint,
                    GithubFetchContext {
                        dependency,
                        registry: context,
                        operation,
                    },
                    reference,
                )
                .await;
        }
        let CanonicalReference::GitHubActionSha { tag, .. } = reference else {
            return Ok(false);
        };
        let Some(url) = github_tag_ref_url(&endpoint.url, tag) else {
            return Ok(false);
        };
        let body = match self
            .get_text_or_status_with_context(&url, dependency.ecosystem, context, operation)
            .await
        {
            Ok(Some(body)) => body,
            Ok(None) => return Ok(false),
            Err(FetchError::RegistryStatus(status)) if status == "not found" => return Ok(false),
            Err(error) => return Err(error),
        };
        if github_action_reference_is_proven_by_exact_ref(reference, &body) {
            return Ok(true);
        }
        let CanonicalReference::GitHubActionSha { commit, .. } = reference else {
            return Ok(false);
        };
        let mut value: Value = from_str(&body).map_err(|_| invalid_tag_response())?;
        if value.get("ref").and_then(Value::as_str) != Some(format!("refs/tags/{tag}").as_str()) {
            return Ok(false);
        }
        let base = endpoint
            .url
            .strip_suffix("/tags")
            .ok_or_else(invalid_tag_response)?;
        let mut visited = HashSet::new();
        for _ in 0..32 {
            let object = value.get("object").ok_or_else(invalid_tag_response)?;
            let sha = object
                .get("sha")
                .and_then(Value::as_str)
                .ok_or_else(invalid_tag_response)?;
            if sha.len() != 40 || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(invalid_tag_response());
            }
            match object.get("type").and_then(Value::as_str) {
                Some("commit") => return Ok(sha_prefix_matches(commit, sha)),
                Some("tag") => {}
                _ => return Err(invalid_tag_response()),
            }
            if !visited.insert(sha.to_ascii_lowercase()) {
                return Err(FetchError::RegistryStatus(
                    "cyclic GitHub tag reference".to_owned(),
                ));
            }
            let url = format!("{base}/git/tags/{sha}");
            let Some(body) = self
                .get_text_or_status_with_context(&url, dependency.ecosystem, context, operation)
                .await?
            else {
                return Err(invalid_tag_response());
            };
            let resolved: Value = from_str(&body).map_err(|_| invalid_tag_response())?;
            if !resolved
                .get("sha")
                .and_then(Value::as_str)
                .is_some_and(|id| id.eq_ignore_ascii_case(sha))
            {
                return Err(invalid_tag_response());
            }
            value = resolved;
        }
        Err(FetchError::RegistryStatus(
            "GitHub tag reference exceeds 32 objects".to_owned(),
        ))
    }

    async fn fetch_floating_github_action_ref(
        &self,
        endpoint: &RegistryEndpoint,
        fetch: GithubFetchContext<'_>,
        reference: &str,
    ) -> Result<bool, FetchError> {
        let Some(tag_url) = github_tag_ref_url(&endpoint.url, reference) else {
            return Err(unavailable_reference(reference));
        };
        let expected_tag = format!("refs/tags/{reference}");
        match self
            .fetch_github_ref(
                &fetch,
                GithubRefTarget {
                    url: &tag_url,
                    expected_ref: &expected_tag,
                    tag: true,
                },
            )
            .await?
        {
            GithubRefStatus::Proven => return Ok(true),
            GithubRefStatus::Missing => {}
        }

        let head_url = tag_url.replacen("/git/ref/tags/", "/git/ref/heads/", 1);
        let expected_head = format!("refs/heads/{reference}");
        match self
            .fetch_github_ref(
                &fetch,
                GithubRefTarget {
                    url: &head_url,
                    expected_ref: &expected_head,
                    tag: false,
                },
            )
            .await?
        {
            GithubRefStatus::Proven => Ok(true),
            GithubRefStatus::Missing => Err(unavailable_reference(reference)),
        }
    }

    async fn fetch_github_ref(
        &self,
        fetch: &GithubFetchContext<'_>,
        target: GithubRefTarget<'_>,
    ) -> Result<GithubRefStatus, FetchError> {
        let body = match self
            .get_text_or_status_with_context(
                target.url,
                fetch.dependency.ecosystem,
                fetch.registry,
                fetch.operation,
            )
            .await
        {
            Ok(Some(body)) => body,
            Ok(None) => return Ok(GithubRefStatus::Missing),
            Err(FetchError::RegistryStatus(status)) if status == "not found" => {
                return Ok(GithubRefStatus::Missing);
            }
            Err(error) => return Err(error),
        };
        github_ref_status(&body, target.expected_ref, target.tag)
    }
}

struct GithubFetchContext<'a> {
    dependency: &'a Dependency,
    registry: &'a RegistryContext,
    operation: &'a OperationContext,
}

struct GithubRefTarget<'a> {
    url: &'a str,
    expected_ref: &'a str,
    tag: bool,
}

enum GithubRefStatus {
    Proven,
    Missing,
}

fn github_ref_status(
    body: &str,
    expected_ref: &str,
    tag: bool,
) -> Result<GithubRefStatus, FetchError> {
    let value: Value = from_str(body).map_err(|_| invalid_reference_response())?;
    let value = match &value {
        Value::Array(entries) if entries.len() > 1 => {
            return Err(FetchError::RegistryStatus(
                "ambiguous GitHub action reference".to_owned(),
            ));
        }
        Value::Array(entries) if entries.is_empty() => return Ok(GithubRefStatus::Missing),
        Value::Array(entries) => &entries[0],
        value => value,
    };
    if value.get("ref").and_then(Value::as_str) != Some(expected_ref) {
        return Err(invalid_reference_response());
    }
    let object = value.get("object").ok_or_else(invalid_reference_response)?;
    let object_type = object
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(invalid_reference_response)?;
    if object_type != "commit" && !(tag && object_type == "tag") {
        return Err(invalid_reference_response());
    }
    let sha = object
        .get("sha")
        .and_then(Value::as_str)
        .ok_or_else(invalid_reference_response)?;
    if sha.len() != 40 || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid_reference_response());
    }
    Ok(GithubRefStatus::Proven)
}

fn unavailable_reference(reference: &str) -> FetchError {
    FetchError::RegistryStatus(format!(
        "GitHub action reference `{reference}` is unavailable"
    ))
}

fn invalid_reference_response() -> FetchError {
    FetchError::RegistryStatus("invalid GitHub action reference response".to_owned())
}

fn invalid_tag_response() -> FetchError {
    FetchError::RegistryStatus("invalid GitHub tag object response".to_owned())
}
