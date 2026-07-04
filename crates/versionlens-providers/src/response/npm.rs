use semver::Version;
use serde_json::Value;
use versionlens_versions::{build_variants, latest_version_with_prerelease_tags};

use super::common::latest_version_strings;
use super::github::{latest_github_commit, latest_github_tag};

pub(crate) fn latest_npm_version(
    value: &Value,
    requirement: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if value.is_array() {
        return latest_github_tag(value, include_prereleases, prerelease_tags)
            .or_else(|| latest_github_commit(value));
    }

    if let Some(tagged) = npm_dist_tag_version(value, requirement) {
        return Some(tagged);
    }

    if let Some(latest) = npm_latest_dist_tag(value) {
        return Some(latest);
    }

    if include_prereleases
        && let Some(latest) = latest_object_key(value, "versions", true, prerelease_tags)
            .or_else(|| latest_version_strings(value, true, prerelease_tags))
    {
        return Some(latest);
    }

    latest_object_key(value, "versions", false, prerelease_tags)
        .or_else(|| latest_version_strings(value, include_prereleases, prerelease_tags))
}

fn npm_dist_tag_version(value: &Value, requirement: &str) -> Option<String> {
    let requirement = requirement.trim();
    if requirement.is_empty() {
        return None;
    }
    value
        .get("dist-tags")?
        .as_object()?
        .get(requirement)?
        .as_str()
        .map(str::to_owned)
}

fn npm_latest_dist_tag(value: &Value) -> Option<String> {
    value
        .pointer("/dist-tags/latest")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn latest_object_key(
    value: &Value,
    key: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = value.get(key)?.as_object()?;
    latest_version_with_prerelease_tags(
        versions.keys().map(String::as_str),
        include_prereleases,
        prerelease_tags,
    )
}

pub fn npm_build_versions(body: &str, requirement: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };

    if let Some(versions) = value.get("versions").and_then(Value::as_object) {
        return sorted_npm_versions(build_variants(
            requirement,
            versions.keys().map(String::as_str),
        ));
    }

    let Some(versions) = value.get("versions").and_then(Value::as_array) else {
        return Vec::new();
    };

    sorted_npm_versions(build_variants(
        requirement,
        versions.iter().filter_map(Value::as_str),
    ))
}

fn sorted_npm_versions(mut versions: Vec<String>) -> Vec<String> {
    versions.sort_by(
        |left, right| match (Version::parse(left), Version::parse(right)) {
            (Ok(left), Ok(right)) => left.cmp(&right),
            _ => left.cmp(right),
        },
    );
    versions
}

pub fn npm_release_versions(body: &str) -> Vec<String> {
    if let Some(versions) = ordered_version_object_keys(body)
        && !versions.is_empty()
    {
        return sorted_npm_versions(versions);
    }

    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };

    if let Some(versions) = value.get("versions").and_then(Value::as_object) {
        return sorted_npm_versions(versions.keys().map(String::to_owned).collect());
    }

    let Some(versions) = value.get("versions").and_then(Value::as_array) else {
        return Vec::new();
    };

    sorted_npm_versions(
        versions
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
    )
}

fn ordered_version_object_keys(body: &str) -> Option<Vec<String>> {
    let versions_index = body.find("\"versions\"")?;
    let object_start = body[versions_index..].find('{')? + versions_index;
    let mut keys = Vec::new();
    let mut depth = 1_u32;
    let mut in_string = false;
    let mut escaped = false;
    let mut capturing_key = false;
    let mut expecting_key = true;
    let mut key = String::new();

    for char in body[object_start + 1..].chars() {
        if in_string {
            if escaped {
                if capturing_key {
                    key.push(char);
                }
                escaped = false;
                continue;
            }
            if char == '\\' {
                escaped = true;
                continue;
            }
            if char == '"' {
                if capturing_key {
                    keys.push(key);
                    key = String::new();
                }
                in_string = false;
                capturing_key = false;
                continue;
            }
            if capturing_key {
                key.push(char);
            }
            continue;
        }

        match char {
            '"' if depth == 1 && expecting_key => {
                in_string = true;
                capturing_key = true;
            }
            '"' => {
                in_string = true;
            }
            '{' | '[' => {
                depth += 1;
            }
            '}' if depth == 1 => {
                return Some(keys);
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
            }
            ':' if depth == 1 => {
                expecting_key = false;
            }
            ',' if depth == 1 => {
                expecting_key = true;
            }
            _ => {}
        }
    }

    None
}
