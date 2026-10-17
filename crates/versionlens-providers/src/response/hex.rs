use serde_json::Value;
use serde_json::from_str;
use versionlens_model::{
    compare_versions, latest_version_with_prerelease_tags, normalized_version,
};

pub(crate) fn latest_hex_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let releases = value.get("releases")?.as_array()?;

    latest_version_with_prerelease_tags(
        releases.iter().filter_map(hex_release_version),
        include_prereleases,
        prerelease_tags,
    )
}

pub(crate) fn hex_release_versions(body: &str) -> Vec<String> {
    let Ok(value) = from_str::<Value>(body) else {
        return vec![];
    };

    let mut versions = value
        .get("releases")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(hex_release_version)
        .filter_map(normalized_version)
        .collect::<Vec<_>>();

    versions.sort_by(|left, right| {
        compare_versions(left, right).unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
    versions.dedup();
    versions
}

fn hex_release_version(release: &Value) -> Option<&str> {
    release.get("version").and_then(|value| value.as_str())
}
