use super::super::super::version_key::{
    canonical_docker_tag, compare_docker_key, docker_version_key,
};

pub(super) fn latest_ranked_tag<'a>(
    tags: impl Iterator<Item = &'a str>,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    tags.filter_map(|tag| {
        Some((
            docker_version_key(tag, include_prereleases, prerelease_tags)?,
            tag,
        ))
    })
    .max_by(|left, right| compare_docker_key(&left.0, &right.0))
    .map(|(key, tag)| canonical_docker_tag(tag, &key))
}
