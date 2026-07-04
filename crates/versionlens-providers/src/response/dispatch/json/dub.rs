use serde_json::Value;

use super::super::ResponseRequest;
use crate::response::dub::latest_dub_version;

pub(super) fn latest_dub_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_dub_version(
        value,
        request.requirement,
        request.include_prereleases,
        request.prerelease_tags,
    )
}
