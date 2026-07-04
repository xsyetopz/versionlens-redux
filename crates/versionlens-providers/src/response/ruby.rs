use serde_json::Value;
use versionlens_versions::{compare_versions, normalized_version};

use super::{
    common::latest_version_array,
    github::{latest_github_commit, latest_github_tag},
};

pub(crate) fn latest_ruby_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if value.is_array()
        && let Some(latest) = latest_github_tag(value, include_prereleases, prerelease_tags)
    {
        return Some(latest);
    }
    if value.is_array()
        && let Some(latest) = latest_github_commit(value)
    {
        return Some(latest);
    }

    latest_version_array(value, "number", include_prereleases, prerelease_tags)
}

pub(crate) fn ruby_release_versions(body: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };

    let mut versions = value
        .get("versions")
        .unwrap_or(&value)
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            entry
                .as_str()
                .or_else(|| entry.get("number").and_then(Value::as_str))
        })
        .filter_map(normalized_version)
        .collect::<Vec<_>>();

    sort_versions(&mut versions);
    versions.dedup();
    versions
}

fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|left, right| {
        compare_versions(left, right).unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
}
