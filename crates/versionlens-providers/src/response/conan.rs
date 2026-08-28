use serde_json::Value;
use std::cmp::Ordering;

pub(crate) fn latest_conan_version(value: &Value, package: &str) -> Option<String> {
    conan_release_versions_from_value(value, package)
        .into_iter()
        .max_by(|left, right| compare_conan_versions(left, right))
}

fn conan_release_versions_from_value(value: &Value, package: &str) -> Vec<String> {
    let Some(results) = value.get("results").and_then(|value| value.as_array()) else {
        return vec![];
    };

    let mut versions = vec![];
    for reference in results.iter().filter_map(|value| value.as_str()) {
        let Some(version) = conan_reference_version(reference, package) else {
            continue;
        };
        if !versions.iter().any(|candidate| candidate == version) {
            versions.push(version.to_owned());
        }
    }
    versions
}

fn conan_reference_version<'a>(reference: &'a str, package: &str) -> Option<&'a str> {
    let (name, rest) = reference.split_once('/')?;
    if name != package {
        return None;
    }
    let version_end = rest.find(['@', '#']).unwrap_or(rest.len());
    let version = &rest[..version_end];
    (!version.is_empty()).then_some(version)
}

fn compare_conan_versions(left: &str, right: &str) -> Ordering {
    match (crate::parse_semver(left), crate::parse_semver(right)) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => versionlens_versions::compare_numeric_text(left, right, &['.', '-', '+']),
    }
}
