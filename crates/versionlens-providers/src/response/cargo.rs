use serde_json::Value;
use versionlens_versions::{latest_version, latest_version_with_prerelease_tags};

pub(crate) fn latest_cargo_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if include_prereleases {
        if let Some(versions) = cargo_versions(value) {
            return latest_version_with_prerelease_tags(
                versions.iter().filter_map(cargo_version_label),
                true,
                prerelease_tags,
            )
            .or_else(|| latest_cargo_crate_version(value, true, prerelease_tags));
        }
    }

    if let Some(versions) = cargo_versions(value) {
        return latest_version(versions.iter().filter_map(cargo_version_label), false)
            .or_else(|| latest_cargo_crate_version(value, false, prerelease_tags));
    }

    latest_cargo_crate_version(value, include_prereleases, prerelease_tags)
}

fn latest_cargo_crate_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if include_prereleases {
        return value
            .pointer("/crate/max_version")
            .and_then(Value::as_str)
            .and_then(|version| {
                latest_version_with_prerelease_tags([version], true, prerelease_tags)
            })
            .or_else(|| stable_cargo_crate_version(value));
    }

    stable_cargo_crate_version(value).or_else(|| {
        value
            .pointer("/crate/max_version")
            .and_then(Value::as_str)
            .and_then(|version| latest_version([version], false))
    })
}

fn stable_cargo_crate_version(value: &Value) -> Option<String> {
    value
        .pointer("/crate/max_stable_version")
        .and_then(Value::as_str)
        .and_then(|version| latest_version([version], false))
}

fn cargo_versions(value: &Value) -> Option<&Vec<Value>> {
    value.get("versions").unwrap_or(value).as_array()
}

fn cargo_version_label(entry: &Value) -> Option<&str> {
    if let Some(version) = entry.as_str() {
        return Some(version);
    }

    (!crate_is_yanked(entry))
        .then(|| entry.get("num")?.as_str())
        .flatten()
}

fn crate_is_yanked(entry: &Value) -> bool {
    entry
        .get("yanked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}
