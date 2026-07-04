use serde_json::Value;
use versionlens_versions::latest_version_with_prerelease_tags;

pub(crate) fn latest_dub_version(
    value: &Value,
    requirement: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = value
        .get("versions")
        .unwrap_or(value)
        .as_array()?
        .iter()
        .filter_map(dub_version_entry)
        .collect::<Vec<_>>();

    if requirement.starts_with('~') && versions.contains(&requirement) {
        return Some(requirement.to_owned());
    }

    latest_version_with_prerelease_tags(
        versions.iter().copied(),
        include_prereleases,
        prerelease_tags,
    )
    .or_else(|| latest_single_dub_pinned_version(&versions))
}

fn dub_version_entry(entry: &Value) -> Option<&str> {
    entry
        .as_str()
        .or_else(|| entry.get("version").and_then(Value::as_str))
}

fn latest_single_dub_pinned_version(versions: &[&str]) -> Option<String> {
    match versions {
        [version] if version.starts_with('~') => Some(version.to_string()),
        _ => None,
    }
}
