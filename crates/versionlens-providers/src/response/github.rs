use serde_json::Value;
use versionlens_versions::latest_version_with_prerelease_tags;

pub(crate) fn latest_github_tag(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_version_with_prerelease_tags(
        value.as_array()?.iter().filter_map(|entry| {
            entry
                .as_str()
                .filter(|tag| !is_commit_sha(tag))
                .or_else(|| entry.get("name").and_then(Value::as_str))
        }),
        include_prereleases,
        prerelease_tags,
    )
}

pub(crate) fn latest_github_commit(value: &Value) -> Option<String> {
    let first = value.as_array()?.first()?;
    let sha = first
        .as_str()
        .or_else(|| first.get("sha").and_then(Value::as_str))?;
    (!sha.is_empty()).then(|| sha.chars().take(7).collect())
}

fn is_commit_sha(value: &str) -> bool {
    value.len() >= 7 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
