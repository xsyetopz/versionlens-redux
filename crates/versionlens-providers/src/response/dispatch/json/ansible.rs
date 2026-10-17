use serde_json::Value;
use versionlens_model::latest_version_with_prerelease_tags;

use crate::response::dispatch::ResponseRequest;

pub(super) fn latest_ansible_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_version_with_prerelease_tags(
        ansible_versions(value),
        request.include_prereleases,
        request.prerelease_tags,
    )
}

fn ansible_versions(value: &Value) -> impl Iterator<Item = &str> {
    value
        .get("data")
        .or_else(|| value.get("results"))
        .or_else(|| value.get("versions"))
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(ansible_version)
}

fn ansible_version(value: &Value) -> Option<&str> {
    value
        .get("version")
        .or_else(|| value.get("name"))
        .and_then(|value| value.as_str())
        .or_else(|| {
            value
                .get("summary_fields")?
                .get("versions")?
                .as_array()?
                .iter()
                .find_map(|version| version.get("name").and_then(|value| value.as_str()))
        })
}
