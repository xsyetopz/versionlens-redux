use serde_json::from_str;
mod json;
mod yanked;

use serde_json::Value;
use versionlens_versions::{
    VersionDialect, compare_versions_for_dialect, normalized_version_for_dialect,
};

use super::xml::{latest_python_rss_version, python_rss_release_versions};
use json::latest_python_json_version;

pub(crate) fn latest_python_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    from_str::<Value>(body)
        .ok()
        .and_then(|value| latest_python_json_version(&value, include_prereleases, prerelease_tags))
        .or_else(|| latest_python_rss_version(body, include_prereleases, prerelease_tags))
}

pub(crate) fn python_release_versions(body: &str) -> Vec<String> {
    let mut versions = from_str::<Value>(body)
        .ok()
        .map(python_json_release_versions)
        .unwrap_or_else(|| python_rss_release_versions(body));

    sort_python_versions(&mut versions);
    versions.dedup();
    versions
}

fn python_json_release_versions(value: Value) -> Vec<String> {
    let versions = value
        .get("releases")
        .and_then(|releases| releases.as_object())
        .map(|releases| {
            releases
                .keys()
                .filter(|version| !yanked::python_release_is_yanked(&value, version))
                .map(|version| version.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            value
                .get("versions")
                .unwrap_or(&value)
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|value| value.as_str())
                .collect()
        });

    versions
        .into_iter()
        .filter_map(|version| normalized_version_for_dialect(version, VersionDialect::Pep440))
        .collect()
}

fn sort_python_versions(versions: &mut [String]) {
    versions.sort_by(|left, right| {
        compare_versions_for_dialect(left, right, VersionDialect::Pep440)
            .unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
}
