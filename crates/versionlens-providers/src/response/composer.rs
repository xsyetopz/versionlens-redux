use serde_json::Value;
use versionlens_versions::{
    compare_versions, latest_version_with_prerelease_tags, normalized_version,
};

pub(crate) fn latest_composer_version(
    value: &Value,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let package_versions = value.get("packages")?.get(package)?;
    let versions = composer_versions_from_package_value(package_versions);

    latest_version_with_prerelease_tags(
        versions.iter().map(String::as_str),
        include_prereleases,
        prerelease_tags,
    )
}

pub(crate) fn composer_release_versions(body: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };
    let Some(packages) = value.get("packages").and_then(Value::as_object) else {
        return Vec::new();
    };

    let mut versions = packages
        .values()
        .flat_map(composer_versions_from_package_value)
        .collect::<Vec<_>>();

    sort_versions(&mut versions);
    versions.dedup();
    versions
}

fn composer_versions_from_package_value(package_versions: &Value) -> Vec<String> {
    if let Some(versions) = package_versions.as_object() {
        return normalize_composer_versions(versions.keys().map(String::as_str));
    }

    let Some(versions) = package_versions.as_array() else {
        return Vec::new();
    };

    normalize_composer_versions(
        versions
            .iter()
            .filter_map(|entry| entry.get("version")?.as_str()),
    )
}

fn normalize_composer_versions<'a>(versions: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    versions
        .into_iter()
        .filter_map(normalized_version)
        .collect()
}

fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|left, right| {
        compare_versions(left, right).unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
}
