use serde_json::Value;

use super::super::ResponseRequest;
use crate::response::docker::latest_docker_tag;

pub(super) fn latest_docker_json_response(
    value: &Value,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    latest_docker_tag(
        value,
        request.requirement,
        request.include_prereleases,
        request.prerelease_tags,
    )
}
