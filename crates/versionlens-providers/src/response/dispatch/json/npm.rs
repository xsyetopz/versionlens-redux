use serde_json::Value;

use super::super::ResponseRequest;
use crate::response::npm::latest_npm_version;

pub(super) fn latest_npm_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_npm_version(
        value,
        request.requirement,
        request.include_prereleases,
        request.prerelease_tags,
    )
}
