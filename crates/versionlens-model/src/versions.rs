use semver::{Error as SemverError, Version as SemverVersion, VersionReq};
use std::cmp::Ordering;

mod latest;
mod parse;
mod pep440;
mod policy;
mod project;
mod range;

pub use latest::{
    latest_stable, latest_version, latest_version_for_dialect, latest_version_with_prerelease_tags,
};
pub use parse::{normalized_version, normalized_version_for_dialect, strip_version_prefix};
pub use policy::{ProjectVersionBump, UpdateLevel, VersionDialect};
pub use project::{is_prerelease_project_version, next_project_version};
pub use range::{
    build_variants, compare_versions, compare_versions_for_dialect, is_build_update,
    is_dotnet_requirement_parseable, is_newer, is_update_available,
    is_update_available_for_dialect, requirement_has_empty_comparator_intersection,
    requirement_is_parseable, requirement_is_parseable_for_dialect, requirement_satisfies_latest,
    requirement_satisfies_latest_for_dialect, update_level,
};

/// Split a version-bearing tag into its preserved textual prefix and its
/// comparable version.
///
/// This keeps repository-specific tag namespaces such as `release/v1.2.3` or
/// `action-v4` separate from the semantic version used for ordering and update
/// choices.
pub fn version_tag_parts(value: &str) -> Option<(&str, &str)> {
    let value = value.trim();
    for (index, character) in value.char_indices() {
        if !character.is_ascii_digit() {
            continue;
        }
        let candidate = &value[index..];
        if normalized_version(candidate).is_some() {
            return Some((&value[..index], candidate));
        }
    }
    None
}

pub fn composer_requirement_is_parseable(requirement: &str) -> bool {
    let requirement = requirement.trim();
    if requirement.is_empty() || normalized_version(requirement).is_some() {
        return !requirement.is_empty();
    }
    let normalized = requirement
        .split_whitespace()
        .map(strip_version_prefix)
        .collect::<Vec<_>>()
        .join(" ");
    VersionReq::parse(&normalized)
        .or_else(|_| {
            VersionReq::parse(&normalized.split_whitespace().collect::<Vec<_>>().join(", "))
        })
        .is_ok()
}

pub fn parse_semver(value: &str) -> Result<SemverVersion, SemverError> {
    value.parse()
}

pub(crate) fn semver_version(major: u64, minor: u64, patch: u64) -> SemverVersion {
    SemverVersion::new(major, minor, patch)
}

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

#[cfg(test)]
mod tag_tests;
