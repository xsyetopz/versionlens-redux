use serde_json::Value;
use versionlens_model::latest_version_with_prerelease_tags;

use super::github::latest_github_tag;

pub(crate) fn latest_swift_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_swift_registry_release(value, include_prereleases, prerelease_tags)
        .or_else(|| latest_github_tag(value, include_prereleases, prerelease_tags))
}

fn json_is_null(value: &Value) -> bool {
    value.is_null()
}

fn latest_swift_registry_release(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let releases = value.get("releases")?.as_object()?;
    let versions = releases.iter().filter_map(|(version, release)| {
        release
            .get("problem")
            .is_none_or(json_is_null)
            .then_some(version.as_str())
    });
    latest_version_with_prerelease_tags(versions, include_prereleases, prerelease_tags)
}
