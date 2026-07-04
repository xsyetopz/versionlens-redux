use serde_json::Value;

use super::entries::docker_tag_entries;

mod selection;

use selection::{latest_alias_tag, latest_matching_suffix, latest_versioned_tag};

pub(crate) fn latest_docker_tag(
    value: &Value,
    requirement: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let tags = docker_tag_entries(value)
        .iter()
        .map(|entry| (entry.name, entry.digest))
        .collect::<Vec<_>>();

    latest_matching_suffix(&tags, requirement, include_prereleases, prerelease_tags)
        .or_else(|| latest_alias_tag(&tags, include_prereleases, prerelease_tags))
        .or_else(|| latest_versioned_tag(tags, include_prereleases, prerelease_tags))
}
