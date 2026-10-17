use serde_json::Value;
use serde_json::from_str;
use versionlens_model::{compare_versions, normalized_version};

pub(crate) fn dotnet_release_versions(body: &str) -> Vec<String> {
    let Ok(value) = from_str::<Value>(body) else {
        return vec![];
    };
    let Some(versions) = value.get("versions").and_then(|value| value.as_array()) else {
        return vec![];
    };

    let mut releases = versions
        .iter()
        .filter_map(|value| value.as_str())
        .filter_map(normalized_version)
        .collect::<Vec<_>>();

    releases.sort_by(|left, right| {
        compare_versions(left, right).unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
    releases.dedup();
    releases
}
