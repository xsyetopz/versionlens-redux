use serde_json::Value;
use versionlens_versions::latest_version_with_prerelease_tags;

pub(crate) fn latest_version_array(
    value: &Value,
    field: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = value
        .get("versions")
        .unwrap_or(value)
        .as_array()?
        .iter()
        .filter_map(|entry| {
            entry
                .as_str()
                .or_else(|| entry.get(field).and_then(Value::as_str))
        });

    latest_version_with_prerelease_tags(versions, include_prereleases, prerelease_tags)
}

pub(crate) fn latest_version_strings(
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

    latest_version_with_prerelease_tags(versions, include_prereleases, prerelease_tags)
}
