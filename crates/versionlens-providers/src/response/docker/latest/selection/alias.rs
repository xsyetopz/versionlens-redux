use super::DockerTagEntry;
use super::ranked::latest_ranked_tag;

pub(in crate::response::docker::latest) fn latest_alias_tag(
    tags: &[DockerTagEntry<'_>],
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let latest_digest = tags
        .iter()
        .find_map(|(tag, digest)| (*tag == "latest").then_some(*digest).flatten())?;
    latest_matching_digest(tags, latest_digest, include_prereleases, prerelease_tags)
}

fn latest_matching_digest(
    tags: &[DockerTagEntry<'_>],
    latest_digest: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_ranked_tag(
        tags.iter()
            .filter(|(tag, digest)| *tag != "latest" && *digest == Some(latest_digest))
            .map(|(tag, _)| *tag),
        include_prereleases,
        prerelease_tags,
    )
}
