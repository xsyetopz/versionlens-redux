use serde_json::Value;
use versionlens_versions::latest_version_with_prerelease_tags;

use super::yanked::python_release_is_yanked;
use crate::response::common::latest_version_strings;

pub(super) fn latest_python_json_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if include_prereleases
        && let Some(latest) = latest_python_release_key(value, true, prerelease_tags)
    {
        return Some(latest);
    }

    value
        .pointer("/info/version")
        .and_then(Value::as_str)
        .filter(|version| !python_release_is_yanked(value, version))
        .and_then(|version| {
            latest_version_with_prerelease_tags([version], include_prereleases, prerelease_tags)
        })
        .or_else(|| latest_python_release_key(value, false, prerelease_tags))
        .or_else(|| latest_version_strings(value, include_prereleases, prerelease_tags))
}

fn latest_python_release_key(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let releases = value.get("releases")?.as_object()?;
    latest_version_with_prerelease_tags(
        releases
            .keys()
            .filter(|version| !python_release_is_yanked(value, version))
            .map(String::as_str),
        include_prereleases,
        prerelease_tags,
    )
}
