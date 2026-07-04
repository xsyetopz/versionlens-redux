use semver::Version;

pub(super) fn prerelease_allowed(
    version: &Version,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> bool {
    if version.pre.is_empty() {
        return true;
    }
    if !include_prereleases {
        return false;
    }
    if prerelease_tags.is_empty() {
        return true;
    }

    let pre = version.pre.as_str();
    prerelease_tags.iter().any(|tag| {
        let normalized_tag = tag.to_ascii_lowercase();
        pre == normalized_tag
            || pre
                .strip_prefix(normalized_tag.as_str())
                .is_some_and(|rest| rest.starts_with('.'))
    })
}
