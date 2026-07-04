use serde_json::Value;
use versionlens_versions::latest_version_with_prerelease_tags;

use super::common::latest_version_strings;

pub(crate) fn latest_go_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| latest_version_strings(&value, include_prereleases, prerelease_tags))
        .or_else(|| {
            latest_version_with_prerelease_tags(body.lines(), include_prereleases, prerelease_tags)
        })
        .map(normalize_go_version)
}

fn normalize_go_version(version: String) -> String {
    version.replacen("+incompatible", "", 1)
}
