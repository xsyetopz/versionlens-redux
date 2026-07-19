use serde_json::Value;
use versionlens_versions::{VersionDialect, latest_version_for_dialect};

use super::yanked::python_release_is_yanked;

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
        .and_then(|value| value.as_str())
        .filter(|version| !python_release_is_yanked(value, version))
        .and_then(|version| {
            latest_version_for_dialect(
                [version],
                include_prereleases,
                prerelease_tags,
                VersionDialect::Pep440,
            )
        })
        .or_else(|| latest_python_release_key(value, false, prerelease_tags))
        .or_else(|| latest_python_version_strings(value, include_prereleases, prerelease_tags))
}

fn latest_python_version_strings(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = value
        .get("versions")
        .unwrap_or(value)
        .as_array()?
        .iter()
        .filter_map(Value::as_str);
    latest_version_for_dialect(
        versions,
        include_prereleases,
        prerelease_tags,
        VersionDialect::Pep440,
    )
}

fn latest_python_release_key(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let releases = value.get("releases")?.as_object()?;
    latest_version_for_dialect(
        releases
            .keys()
            .filter(|version| !python_release_is_yanked(value, version))
            .map(|value| value.as_str()),
        include_prereleases,
        prerelease_tags,
        VersionDialect::Pep440,
    )
}
