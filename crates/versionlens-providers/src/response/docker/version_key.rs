use std::cmp::Ordering;

pub(super) fn docker_version_key(
    tag: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<Vec<u64>> {
    let (version, suffix) = tag
        .split_once('-')
        .map_or((tag, ""), |(version, suffix)| (version, suffix));
    if is_docker_prerelease_suffix(suffix) {
        if !include_prereleases {
            return None;
        }
        if !prerelease_tags.is_empty()
            && !prerelease_tags
                .iter()
                .any(|tag| docker_prerelease_suffix_matches(suffix, tag))
        {
            return None;
        }
    }
    let first_part = version.split('.').next()?;
    if first_part.len() > 4 {
        return None;
    }
    let parts = version
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (!parts.is_empty() && parts.len() <= 3).then_some(parts)
}

pub(super) fn canonical_docker_tag(tag: &str, key: &[u64]) -> String {
    if tag.contains('-') {
        tag.to_owned()
    } else {
        key.iter().map(u64::to_string).collect::<Vec<_>>().join(".")
    }
}

fn is_docker_prerelease_suffix(suffix: &str) -> bool {
    ["alpha", "beta", "preview", "rc"]
        .iter()
        .any(|tag| docker_prerelease_suffix_matches(suffix, tag))
}

fn docker_prerelease_suffix_matches(suffix: &str, tag: &str) -> bool {
    suffix == tag
        || suffix
            .strip_prefix(tag)
            .is_some_and(|rest| rest.starts_with('.') || rest.starts_with('-'))
}

pub(super) fn compare_docker_key(left: &[u64], right: &[u64]) -> Ordering {
    let len = left.len().max(right.len());
    let number_ordering = (0..len)
        .map(|index| {
            left.get(index)
                .unwrap_or(&0)
                .cmp(right.get(index).unwrap_or(&0))
        })
        .find(|ordering| *ordering != Ordering::Equal)
        .unwrap_or(Ordering::Equal);

    if number_ordering == Ordering::Equal {
        left.len().cmp(&right.len())
    } else {
        number_ordering
    }
}
