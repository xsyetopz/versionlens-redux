use std::collections::BTreeSet;

use serde_json::Value;
use versionlens_model::latest_version_with_prerelease_tags;

use crate::response::dispatch::ResponseRequest;

pub(super) fn latest_bazel_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    let yanked = yanked_versions(value);
    latest_version_with_prerelease_tags(
        value
            .get("versions")?
            .as_array()?
            .iter()
            .filter_map(|value| value.as_str())
            .filter(|version| !yanked.contains(*version)),
        request.include_prereleases,
        request.prerelease_tags,
    )
}

fn yanked_versions(value: &Value) -> BTreeSet<&str> {
    value
        .get("yanked_versions")
        .and_then(|value| value.as_object())
        .map(|versions| versions.keys().map(|value| value.as_str()).collect())
        .unwrap_or_default()
}
