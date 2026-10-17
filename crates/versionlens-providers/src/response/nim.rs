use serde_json::Value;
use versionlens_model::latest_version_with_prerelease_tags;

use super::github::latest_github_tag;

pub(crate) fn latest_nim_version(
    value: &Value,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_github_tag(value, include_prereleases, prerelease_tags).or_else(|| {
        latest_nim_package_list_version(value, package, include_prereleases, prerelease_tags)
    })
}

fn latest_nim_package_list_version(
    value: &Value,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let package = value
        .as_array()?
        .iter()
        .find(|entry| entry.get("name").and_then(|value| value.as_str()) == Some(package))?;

    let versions = package
        .get("versions")
        .or_else(|| package.get("releases"))?
        .as_array()?
        .iter()
        .filter_map(|entry| {
            entry
                .as_str()
                .or_else(|| entry.get("version").and_then(|value| value.as_str()))
                .or_else(|| entry.get("tag").and_then(|value| value.as_str()))
        });

    latest_version_with_prerelease_tags(versions, include_prereleases, prerelease_tags)
}
