use serde_json::Value;

use super::super::ResponseRequest;
use crate::response::cargo::latest_cargo_version;

pub(super) fn latest_cargo_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_cargo_version(value, request.include_prereleases, request.prerelease_tags)
}
