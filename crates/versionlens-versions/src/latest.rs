use semver::Version;

use crate::VersionDialect;
use crate::parse::parse_version;
use crate::pep440;

mod prerelease;

use prerelease::prerelease_allowed;

pub fn latest_stable<'a>(versions: impl IntoIterator<Item = &'a str>) -> Option<String> {
    latest_version(versions, false)
}

pub fn latest_version<'a>(
    versions: impl IntoIterator<Item = &'a str>,
    include_prereleases: bool,
) -> Option<String> {
    latest_version_with_prerelease_tags(versions, include_prereleases, &[])
}

pub fn latest_version_with_prerelease_tags<'a>(
    versions: impl IntoIterator<Item = &'a str>,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    versions
        .into_iter()
        .filter_map(|raw| parse_version_entry(raw, include_prereleases, prerelease_tags))
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, raw)| raw.to_owned())
}

pub fn latest_version_for_dialect<'a>(
    versions: impl IntoIterator<Item = &'a str>,
    include_prereleases: bool,
    prerelease_tags: &[String],
    dialect: VersionDialect,
) -> Option<String> {
    if dialect == VersionDialect::Semver {
        return latest_version_with_prerelease_tags(versions, include_prereleases, prerelease_tags);
    }

    versions
        .into_iter()
        .filter_map(|raw| {
            let version = pep440::parse_version(raw)?;
            pep440::prerelease_allowed(&version, include_prereleases, prerelease_tags)
                .then_some((version, raw))
        })
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, raw)| raw.to_owned())
}

fn parse_version_entry<'a>(
    raw: &'a str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<(Version, &'a str)> {
    if has_four_numeric_segments(raw) {
        return None;
    }
    let version = parse_version(raw)?;
    prerelease_allowed(&version, include_prereleases, prerelease_tags).then_some((version, raw))
}

fn has_four_numeric_segments(raw: &str) -> bool {
    let Some(start) = raw.find(|char: char| char.is_ascii_digit()) else {
        return false;
    };
    let numeric = &raw[start..];
    let core_end = numeric.find(['-', '+']).unwrap_or(numeric.len());
    let core = &numeric[..core_end];
    let mut segments = 0;
    for part in core.split('.') {
        if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
        segments += 1;
        if segments >= 4 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests;
