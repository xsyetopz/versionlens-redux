use serde_json::Value;
use versionlens_versions::latest_version_with_prerelease_tags;

pub(crate) fn latest_deno_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if include_prereleases
        && let Some(latest) = latest_deno_version_key(value, true, prerelease_tags)
    {
        return Some(latest);
    }

    value
        .get("latest")
        .and_then(Value::as_str)
        .filter(|version| !deno_release_is_yanked(value, version))
        .and_then(|version| {
            latest_version_with_prerelease_tags([version], include_prereleases, prerelease_tags)
        })
        .or_else(|| latest_deno_version_key(value, false, prerelease_tags))
}

fn latest_deno_version_key(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if let Some(versions) = value.get("versions").or(Some(value))?.as_array() {
        return latest_version_with_prerelease_tags(
            versions.iter().filter_map(Value::as_str),
            include_prereleases,
            prerelease_tags,
        );
    }

    let versions = value.get("versions")?.as_object()?;

    latest_version_with_prerelease_tags(
        versions
            .iter()
            .filter(|(_, entry)| !deno_version_is_yanked(entry))
            .map(|(version, _)| version.as_str()),
        include_prereleases,
        prerelease_tags,
    )
}

fn deno_release_is_yanked(value: &Value, version: &str) -> bool {
    value
        .get("versions")
        .and_then(Value::as_object)
        .and_then(|versions| versions.get(version))
        .is_some_and(deno_version_is_yanked)
}

fn deno_version_is_yanked(entry: &Value) -> bool {
    entry
        .get("yanked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}
