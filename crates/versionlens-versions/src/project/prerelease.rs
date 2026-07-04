use semver::Version;

pub(super) fn release_version(mut version: Version) -> Version {
    version.pre = semver::Prerelease::EMPTY;
    version
}

pub(super) fn increment_prerelease(mut version: Version) -> Option<Version> {
    let pre = version.pre.as_str();
    let next_pre = match pre.rsplit_once('.') {
        Some((prefix, suffix)) => suffix
            .parse::<u64>()
            .ok()
            .map(|number| format!("{prefix}.{}", number + 1))
            .unwrap_or_else(|| format!("{pre}.0")),
        None if pre.is_empty() => "pre.0".to_owned(),
        None => format!("{pre}.0"),
    };
    version.pre = semver::Prerelease::new(&next_pre).ok()?;
    Some(version)
}
