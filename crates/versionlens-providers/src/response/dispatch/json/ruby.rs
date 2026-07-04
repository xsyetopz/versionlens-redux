use serde_json::Value;

use super::super::ResponseRequest;
use crate::response::ruby::latest_ruby_version;

pub(super) fn latest_ruby_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_ruby_version(value, request.include_prereleases, request.prerelease_tags)
}
