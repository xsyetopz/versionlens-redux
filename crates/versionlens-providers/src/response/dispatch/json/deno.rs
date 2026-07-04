use serde_json::Value;

use super::super::ResponseRequest;
use crate::response::deno::latest_deno_version;

pub(super) fn latest_deno_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_deno_version(value, request.include_prereleases, request.prerelease_tags)
}
