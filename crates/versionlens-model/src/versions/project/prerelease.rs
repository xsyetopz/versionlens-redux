use semver::Prerelease;
use semver::Version;

fn parse_prerelease(value: &str) -> Option<Prerelease> {
    value.parse().ok()
}

pub(super) fn release_version(mut version: Version) -> Version {
    if let Some(pre) = parse_prerelease("") {
        version.pre = pre;
    }
    version
}

pub(super) fn increment_prerelease(mut version: Version) -> Option<Version> {
    let pre = version.pre.as_str();
    let next_pre = match pre.rsplit_once('.') {
        Some((prefix, suffix)) => suffix
            .parse::<u64>()
            .ok()
            .and_then(|number| number.checked_add(1))
            .map(|number| format!("{prefix}.{number}"))
            .unwrap_or_else(|| format!("{pre}.0")),
        None if pre.is_empty() => "pre.0".to_owned(),
        None => format!("{pre}.0"),
    };
    version.pre = parse_prerelease(&next_pre)?;
    Some(version)
}
