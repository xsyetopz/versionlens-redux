use super::DockerTagEntry;
use super::ranked::latest_ranked_tag;

pub(in crate::response::docker::latest) fn latest_versioned_tag(
    tags: Vec<DockerTagEntry<'_>>,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_ranked_tag(
        tags.into_iter().map(|(tag, _)| tag),
        include_prereleases,
        prerelease_tags,
    )
}
