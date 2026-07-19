use serde_json::Value;
use serde_json::Value::{
    Array as JsonValueArray, Bool as JsonValueBool, Null as JsonValueNull,
    String as JsonValueString,
};
use serde_json::from_str;
use versionlens_versions::{latest_version_with_prerelease_tags, normalized_version};

pub(crate) fn latest_go_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    from_str::<Value>(body)
        .ok()
        .and_then(|value| latest_go_metadata_versions(&value, include_prereleases, prerelease_tags))
        .or_else(|| latest_go_module_proxy_version(body.lines().collect(), prerelease_tags))
        .map(normalize_go_version)
}

pub(crate) fn latest_go_module_list_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = body
        .lines()
        .map(str::trim)
        .filter(|version| !version.is_empty())
        .collect::<Vec<_>>();
    if versions.is_empty()
        || versions
            .iter()
            .any(|version| normalized_version(version).is_none())
    {
        return None;
    }
    let prerelease_tags = if include_prereleases {
        prerelease_tags
    } else {
        &[]
    };
    latest_go_module_proxy_version(versions, prerelease_tags).map(normalize_go_version)
}

pub(crate) fn latest_go_module_latest_version(body: &str) -> Option<String> {
    let value = from_str::<Value>(body).ok()?;
    let version = go_metadata_entry_version(&value)?;
    if go_metadata_entry_is_unavailable(&value) || normalized_version(version).is_none() {
        return None;
    }
    Some(normalize_go_version(version.to_owned()))
}

fn latest_go_metadata_versions(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if let Some(version) = go_metadata_entry_version(value) {
        return if go_metadata_entry_is_unavailable(value) {
            None
        } else {
            Some(version.to_owned())
        };
    }

    let versions = value
        .get("versions")
        .unwrap_or(value)
        .as_array()?
        .iter()
        .filter_map(|entry| {
            if go_metadata_entry_is_unavailable(entry) {
                return None;
            }

            go_metadata_entry_version(entry)
        });

    let versions = versions.collect();
    if include_prereleases {
        latest_go_module_proxy_version(versions, prerelease_tags)
    } else {
        latest_go_module_proxy_version(versions, &[])
    }
}

fn latest_go_module_proxy_version(
    versions: Vec<&str>,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_version_with_prerelease_tags(
        versions
            .iter()
            .copied()
            .filter(|version| !is_go_pseudo_version(version)),
        false,
        prerelease_tags,
    )
    .or_else(|| {
        latest_version_with_prerelease_tags(
            versions
                .iter()
                .copied()
                .filter(|version| !is_go_pseudo_version(version)),
            true,
            prerelease_tags,
        )
    })
    .or_else(|| latest_go_pseudo_version(versions))
}

fn latest_go_pseudo_version(versions: Vec<&str>) -> Option<String> {
    versions
        .into_iter()
        .filter_map(|version| {
            go_pseudo_version_timestamp(version).map(|timestamp| (timestamp, version))
        })
        .max_by(|left, right| left.0.cmp(right.0))
        .map(|(_, version)| version.to_owned())
}

fn is_go_pseudo_version(version: &str) -> bool {
    go_pseudo_version_timestamp(version).is_some()
}

fn go_pseudo_version_timestamp(version: &str) -> Option<&str> {
    normalized_version(version)?;
    let version = version.strip_prefix('v').unwrap_or(version);
    let mut parts = version.rsplit('-');
    parts.next()?;
    let timestamp_part = parts.next()?;
    let timestamp = timestamp_part
        .rsplit_once('.')
        .map_or(timestamp_part, |(_, timestamp)| timestamp);
    (timestamp.len() == 14 && timestamp.bytes().all(|byte| byte.is_ascii_digit()))
        .then_some(timestamp)
}

fn go_metadata_entry_version(entry: &Value) -> Option<&str> {
    entry
        .as_str()
        .or_else(|| entry.get("Version").and_then(|value| value.as_str()))
        .or_else(|| entry.get("version").and_then(|value| value.as_str()))
}

fn go_metadata_entry_is_unavailable(entry: &Value) -> bool {
    go_version_is_retracted(entry) || go_module_is_deprecated(entry)
}

fn go_module_is_deprecated(entry: &Value) -> bool {
    entry
        .get("Deprecated")
        .or_else(|| entry.get("deprecated"))
        .is_some_and(go_deprecated_value_is_active)
}

fn go_deprecated_value_is_active(value: &Value) -> bool {
    match value {
        JsonValueNull | JsonValueBool(false) => false,
        JsonValueString(message) => !message.trim().is_empty(),
        JsonValueArray(items) => !items.is_empty(),
        _ => true,
    }
}

fn go_version_is_retracted(entry: &Value) -> bool {
    entry
        .get("Retracted")
        .or_else(|| entry.get("retracted"))
        .is_some_and(go_retracted_value_is_active)
}

fn go_retracted_value_is_active(value: &Value) -> bool {
    match value {
        JsonValueNull | JsonValueBool(false) => false,
        JsonValueArray(items) => !items.is_empty(),
        _ => true,
    }
}

fn normalize_go_version(version: String) -> String {
    version.replacen("+incompatible", "", 1)
}
