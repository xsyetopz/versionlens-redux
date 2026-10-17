use serde_json::Value;
use versionlens_model::latest_version_with_prerelease_tags;

use super::super::ResponseRequest;

pub(super) fn latest_cocoapods_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    let versions = value
        .get("versions")?
        .as_array()?
        .iter()
        .filter_map(|entry| {
            entry
                .as_str()
                .or_else(|| entry.get("name").and_then(|value| value.as_str()))
                .or_else(|| entry.get("version").and_then(|value| value.as_str()))
        });

    latest_version_with_prerelease_tags(
        versions,
        request.include_prereleases,
        request.prerelease_tags,
    )
}
