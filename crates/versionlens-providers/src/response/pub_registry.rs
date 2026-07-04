use serde_json::Value;
use versionlens_versions::latest_version_with_prerelease_tags;

pub(crate) fn latest_pub_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if let Some(versions) = value.get("versions").and_then(Value::as_array) {
        return latest_version_with_prerelease_tags(
            versions
                .iter()
                .filter(|entry| !pub_version_is_retracted(entry))
                .filter_map(pub_version_entry),
            include_prereleases,
            prerelease_tags,
        )
        .or_else(|| latest_pub_alias_version(value, include_prereleases, prerelease_tags));
    }

    latest_pub_alias_version(value, include_prereleases, prerelease_tags)
}

fn latest_pub_alias_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    value
        .pointer("/latest/version")
        .and_then(Value::as_str)
        .and_then(|version| {
            latest_version_with_prerelease_tags([version], include_prereleases, prerelease_tags)
        })
}

fn pub_version_entry(entry: &Value) -> Option<&str> {
    entry
        .as_str()
        .or_else(|| entry.get("version").and_then(Value::as_str))
}

fn pub_version_is_retracted(entry: &Value) -> bool {
    entry
        .get("retracted")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}
