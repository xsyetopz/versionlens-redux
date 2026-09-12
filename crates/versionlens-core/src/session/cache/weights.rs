use super::CachedLatest;
use crate::vulnerability::VulnerabilityCheck;
use versionlens_model::{CanonicalReference, Dependency};
use versionlens_suggestions::{Suggestion, UpdateChoice};

fn optional(value: Option<&String>) -> usize {
    value.map_or(0, String::capacity)
}

fn vector<T>(values: &Vec<T>, heap: fn(&T) -> usize) -> usize {
    values.capacity() * size_of::<T>() + values.iter().map(heap).sum::<usize>()
}

fn choice(value: &UpdateChoice) -> usize {
    value.label.capacity()
        + value.version.capacity()
        + value.command.capacity()
        + optional(value.replacement.as_ref())
}

pub(in crate::session) fn latest(value: &CachedLatest) -> usize {
    value.latest.capacity()
        + vector(&value.builds, String::capacity)
        + vector(&value.choices, choice)
}

pub(in crate::session) fn suggestion(value: &Suggestion) -> usize {
    dependency(&value.dependency)
        + optional(value.latest.as_ref())
        + optional(value.resolved.as_ref())
        + vector(&value.builds, String::capacity)
        + vector(&value.choices, choice)
}

pub(in crate::session) fn dependencies(values: &Vec<Dependency>) -> usize {
    vector(values, dependency)
}

fn dependency(dependency: &Dependency) -> usize {
    let canonical = match &dependency.canonical_reference {
        Some(CanonicalReference::GitHubActionTag { tag }) => tag.capacity(),
        Some(CanonicalReference::GitHubActionCommit { commit }) => commit.capacity(),
        Some(CanonicalReference::GitHubActionRef { reference })
        | Some(CanonicalReference::GitHubActionInvalid { reference }) => reference.capacity(),
        Some(CanonicalReference::GitHubActionLocal { path, .. }) => path.capacity(),
        Some(CanonicalReference::GitHubActionExpression { expression }) => expression.capacity(),
        Some(CanonicalReference::GitHubActionSha {
            commit,
            tag,
            separator,
        }) => commit.capacity() + tag.capacity() + separator.capacity(),
        None => 0,
    };
    dependency.name.capacity()
        + dependency.requirement.capacity()
        + dependency.group.capacity()
        + dependency.requirement_prefix.capacity()
        + dependency.requirement_suffix.capacity()
        + optional(dependency.hosted_url.as_ref())
        + optional(dependency.hosted_name.as_ref())
        + canonical
}

pub(in crate::session) fn vulnerability(value: &VulnerabilityCheck) -> usize {
    match value {
        VulnerabilityCheck::Checked(advisories) => vector(advisories, |advisory| {
            advisory.id.capacity() + advisory.title.capacity() + optional(advisory.url.as_ref())
        }),
        VulnerabilityCheck::Failed(message) => message.capacity(),
        VulnerabilityCheck::Unsupported => 0,
    }
}
