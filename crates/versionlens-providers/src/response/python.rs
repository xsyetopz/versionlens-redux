mod json;
mod yanked;

use serde_json::Value;
use versionlens_versions::{compare_versions, normalized_version};

use super::xml::latest_python_rss_version;
use json::latest_python_json_version;

pub(crate) fn latest_python_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| latest_python_json_version(&value, include_prereleases, prerelease_tags))
        .or_else(|| latest_python_rss_version(body, include_prereleases, prerelease_tags))
}

pub(crate) fn python_release_versions(body: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };

    let mut versions = value
        .get("versions")
        .unwrap_or(&value)
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(normalized_version)
        .collect::<Vec<_>>();

    sort_python_versions(&mut versions);
    versions.dedup();
    versions
}

fn sort_python_versions(versions: &mut [String]) {
    versions.sort_by(|left, right| {
        compare_versions(left, right).unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
}
