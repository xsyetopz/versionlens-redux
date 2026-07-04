use std::cmp::Ordering;

use versionlens_versions::{
    compare_versions, latest_version_with_prerelease_tags, normalized_version,
};

mod collect;

use collect::collect_element_texts;

pub(crate) fn latest_python_rss_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = collect_element_texts(body, b"title", python_rss_title_version)?;
    latest_from_versions(versions, include_prereleases, prerelease_tags)
        .and_then(|version| normalized_version(&version))
}

pub(crate) fn latest_maven_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = collect_element_texts(body, b"version", |version| Some(version.to_owned()))?;
    latest_from_versions(versions, include_prereleases, prerelease_tags)
}

pub(crate) fn maven_release_versions(body: &str) -> Vec<String> {
    let Some(mut versions) = collect_element_texts(body, b"version", version_is_semver) else {
        return Vec::new();
    };

    versions.sort_by(|left, right| compare_versions(left, right).unwrap_or(Ordering::Equal));
    versions.dedup();
    versions
}

fn python_rss_title_version(title: &str) -> Option<String> {
    let version = title.rsplit_once(' ').map_or(title, |(_, version)| version);
    version
        .chars()
        .next()
        .is_some_and(|char| char.is_ascii_digit())
        .then(|| version.to_owned())
}

fn version_is_semver(version: &str) -> Option<String> {
    normalized_version(version)
        .is_some()
        .then(|| version.to_owned())
}

fn latest_from_versions(
    versions: Vec<String>,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_version_with_prerelease_tags(
        versions.iter().map(String::as_str),
        include_prereleases,
        prerelease_tags,
    )
}
