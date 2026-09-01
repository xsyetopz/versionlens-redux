mod latest;
mod parse;
mod pep440;
mod policy;
mod project;
mod range;
mod support;

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
pub use support::{compare_numeric_segments, compare_numeric_text, numeric_segments};

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
    parse_semver_req(&normalized)
        .or_else(|_| {
            parse_semver_req(&normalized.split_whitespace().collect::<Vec<_>>().join(", "))
        })
        .is_ok()
}
pub(crate) use support::{parse_semver, parse_semver_req, semver_version};

#[cfg(test)]
mod tag_tests;
