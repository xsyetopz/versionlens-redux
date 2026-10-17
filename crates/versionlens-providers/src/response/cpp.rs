use serde_json::Value;
use versionlens_model::latest_version_with_prerelease_tags;

use super::github::latest_github_tag;

pub(crate) fn latest_cpp_json_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_github_tag(value, include_prereleases, prerelease_tags)
}

pub(crate) fn latest_cpp_text_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_version_with_prerelease_tags(xmake_versions(body), include_prereleases, prerelease_tags)
}

fn xmake_versions(body: &str) -> impl Iterator<Item = &str> {
    body.match_indices("add_versions(")
        .filter_map(|(start, _)| {
            let rest = &body[start + "add_versions(".len()..];
            let quote = rest
                .bytes()
                .position(|byte| byte == b'\'' || byte == b'\"')?;
            let quote_byte = rest.as_bytes()[quote];
            let value_start = quote + 1;
            let value_end = rest[value_start..]
                .bytes()
                .position(|byte| byte == quote_byte)?
                + value_start;
            Some(&rest[value_start..value_end])
        })
}
