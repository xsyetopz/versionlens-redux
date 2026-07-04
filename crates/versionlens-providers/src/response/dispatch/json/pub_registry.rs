use serde_json::Value;

use super::super::ResponseRequest;
use crate::response::pub_registry::latest_pub_version;

pub(super) fn latest_pub_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_pub_version(value, request.include_prereleases, request.prerelease_tags)
}
