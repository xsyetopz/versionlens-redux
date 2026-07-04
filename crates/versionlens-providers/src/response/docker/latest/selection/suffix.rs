use super::DockerTagEntry;
use super::ranked::latest_ranked_tag;

pub(in crate::response::docker::latest) fn latest_matching_suffix(
    tags: &[DockerTagEntry<'_>],
    requirement: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let requirement_suffix = docker_tag_suffix(requirement)?;
    latest_ranked_tag(
        tags.iter()
            .map(|(tag, _)| *tag)
            .filter(|tag| docker_tag_suffix(tag) == Some(requirement_suffix)),
        include_prereleases,
        prerelease_tags,
    )
}

fn docker_tag_suffix(tag: &str) -> Option<&str> {
    tag.split_once('-')
        .map(|(_, suffix)| suffix)
        .filter(|suffix| !suffix.is_empty())
}
