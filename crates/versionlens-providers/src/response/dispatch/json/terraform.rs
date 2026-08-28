use crate::response::dispatch::ResponseRequest;
use crate::response::versions::object_version_values;
use serde_json::Value;

pub(super) fn latest_terraform_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    let versions = object_version_values(value)?;
    versionlens_versions::latest_version_with_prerelease_tags(
        versions,
        request.include_prereleases,
        request.prerelease_tags,
    )
}
