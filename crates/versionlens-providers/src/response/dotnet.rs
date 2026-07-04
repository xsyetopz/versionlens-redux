use serde_json::Value;
use versionlens_versions::{compare_versions, normalized_version};

pub(crate) fn dotnet_release_versions(body: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };
    let Some(versions) = value.get("versions").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut releases = versions
        .iter()
        .filter_map(Value::as_str)
        .filter_map(normalized_version)
        .collect::<Vec<_>>();

    releases.sort_by(|left, right| {
        compare_versions(left, right).unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
    releases.dedup();
    releases
}
