use serde_json::Value;
use serde_json::from_str;
use versionlens_versions::{
    compare_versions, latest_version_with_prerelease_tags, normalized_version,
};

pub(crate) fn latest_composer_version(
    value: &Value,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let package_versions = composer_package_versions_value(value, package)?;
    let versions = composer_versions_from_package_value(package_versions);

    latest_version_with_prerelease_tags(
        versions.iter().map(|value| value.as_str()),
        include_prereleases,
        prerelease_tags,
    )
}

pub(crate) fn composer_release_versions(body: &str) -> Vec<String> {
    let Ok(value) = from_str::<Value>(body) else {
        return vec![];
    };

    let mut versions = composer_package_version_values(&value)
        .into_iter()
        .flat_map(composer_versions_from_package_value)
        .collect::<Vec<_>>();

    sort_versions(&mut versions);
    versions.dedup();
    versions
}

/// Extract releases only for `package` from a Composer response.
///
/// Unlike the compatibility extractor above, this is safe for Composer
/// responses whose `packages` object contains metadata for multiple packages.
pub(crate) fn composer_release_versions_for_package(body: &str, package: &str) -> Vec<String> {
    let Ok(value) = from_str::<Value>(body) else {
        return vec![];
    };

    let Some(package_versions) = composer_package_versions_value(&value, package) else {
        return vec![];
    };

    let mut versions = composer_versions_from_package_value(package_versions);
    sort_versions(&mut versions);
    versions.dedup();
    versions
}

fn composer_package_versions_value<'a>(value: &'a Value, package: &str) -> Option<&'a Value> {
    value
        .get("packages")
        .and_then(|packages| packages.get(package))
        .or_else(|| composer_json_api_package_versions(value, package))
}

fn composer_package_version_values(value: &Value) -> Vec<&Value> {
    let mut versions = value
        .get("packages")
        .and_then(|value| value.as_object())
        .into_iter()
        .flat_map(|packages| packages.values())
        .collect::<Vec<_>>();
    if let Some(package_versions) = value
        .get("package")
        .and_then(|package| package.get("versions"))
    {
        versions.push(package_versions);
    }
    versions
}

fn composer_json_api_package_versions<'a>(value: &'a Value, package: &str) -> Option<&'a Value> {
    let package_value = value.get("package")?;
    if package_value
        .get("name")
        .and_then(|value| value.as_str())
        .is_some_and(|name| name != package)
    {
        return None;
    }
    package_value.get("versions")
}

fn composer_versions_from_package_value(package_versions: &Value) -> Vec<String> {
    if let Some(versions) = package_versions.as_object() {
        let mut output = normalize_composer_versions(versions.keys().map(|value| value.as_str()));
        output.extend(versions.values().flat_map(composer_branch_aliases));
        return output;
    }

    let Some(versions) = package_versions.as_array() else {
        return vec![];
    };

    let mut output = normalize_composer_versions(
        versions
            .iter()
            .filter_map(|entry| entry.get("version")?.as_str()),
    );
    output.extend(versions.iter().flat_map(composer_branch_aliases));
    output
}

fn normalize_composer_versions<'a>(versions: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    versions
        .into_iter()
        .filter_map(normalized_version)
        .collect()
}

fn composer_branch_aliases(value: &Value) -> Vec<String> {
    value
        .get("extra")
        .and_then(|extra| extra.get("branch-alias"))
        .and_then(|value| value.as_object())
        .into_iter()
        .flat_map(|aliases| aliases.values())
        .filter_map(|value| value.as_str())
        .filter(|alias| is_composer_branch_alias(alias))
        .map(|value| value.to_owned())
        .collect()
}

fn is_composer_branch_alias(alias: &str) -> bool {
    let alias = alias.trim();
    alias.ends_with(".x-dev")
        && alias
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_digit())
}

fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|left, right| {
        compare_versions(left, right).unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
}
