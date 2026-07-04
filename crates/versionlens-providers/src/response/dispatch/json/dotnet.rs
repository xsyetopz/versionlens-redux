use serde_json::Value;

use super::super::ResponseRequest;
use crate::response::common::latest_version_strings;

pub(super) fn latest_dotnet_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_version_strings(value, request.include_prereleases, request.prerelease_tags)
}
