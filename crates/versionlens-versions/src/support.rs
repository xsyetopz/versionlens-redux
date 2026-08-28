use semver::{
    BuildMetadata as SemverBuildMetadata, Error as SemverError, Prerelease as SemverPrerelease,
    Version as SemverVersion, VersionReq as SemverVersionReq,
};

pub(crate) fn parse_semver(value: &str) -> Result<SemverVersion, SemverError> {
    value.parse()
}

fn empty_prerelease() -> SemverPrerelease {
    SemverPrerelease::EMPTY
}

fn empty_build_metadata() -> SemverBuildMetadata {
    SemverBuildMetadata::EMPTY
}

pub(crate) fn semver_version(major: u64, minor: u64, patch: u64) -> SemverVersion {
    SemverVersion {
        major,
        minor,
        patch,
        pre: empty_prerelease(),
        build: empty_build_metadata(),
    }
}

pub(crate) fn parse_semver_req(value: &str) -> Result<SemverVersionReq, SemverError> {
    value.parse()
}
use std::cmp::Ordering;

pub fn numeric_segments(tag: &str) -> Option<Vec<u64>> {
    let version = tag.split_once('-').map_or(tag, |(version, _)| version);
    let numbers = version
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (!numbers.is_empty()).then_some(numbers)
}

pub fn compare_numeric_segments(left: &[u64], right: &[u64]) -> Ordering {
    let len = left.len().max(right.len());
    (0..len)
        .map(|index| {
            left.get(index)
                .unwrap_or(&0)
                .cmp(right.get(index).unwrap_or(&0))
        })
        .find(|ordering| *ordering != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

pub fn compare_numeric_text(left: &str, right: &str, separators: &[char]) -> Ordering {
    let left = left
        .split(|character| separators.contains(&character))
        .map(|segment| segment.parse::<u64>().unwrap_or(0))
        .collect::<Vec<_>>();
    let right = right
        .split(|character| separators.contains(&character))
        .map(|segment| segment.parse::<u64>().unwrap_or(0))
        .collect::<Vec<_>>();
    compare_numeric_segments(&left, &right).then_with(|| left.len().cmp(&right.len()))
}
