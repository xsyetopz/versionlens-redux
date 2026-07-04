use serde_json::Value;

use super::super::ResponseRequest;
use crate::response::composer::latest_composer_version;

pub(super) fn latest_composer_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_composer_version(
        value,
        request.package,
        request.include_prereleases,
        request.prerelease_tags,
    )
}
