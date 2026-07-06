use semver::VersionReq;

use crate::parse::{normalize_requirement, parse_version};

pub(in crate::range) fn semver_range_requirement_satisfies(
    requirement: &str,
    latest: &str,
) -> Option<bool> {
    if !looks_like_range(requirement) {
        return None;
    }

    let Some(latest) = parse_version(latest) else {
        return None;
    };
    parse_normalized_version_req(requirement).map(|requirement| requirement.matches(&latest))
}

fn parse_normalized_version_req(requirement: &str) -> Option<VersionReq> {
    let normalized = normalize_requirement(requirement);
    crate::parse_semver_req(&normalized)
        .or_else(|_| {
            crate::parse_semver_req(&normalized.split_whitespace().collect::<Vec<_>>().join(", "))
        })
        .ok()
}

pub(in crate::range::requirements) fn looks_like_range(requirement: &str) -> bool {
    if requirement.trim().eq_ignore_ascii_case("any") {
        return true;
    }

    requirement
        .chars()
        .any(|char| matches!(char, '^' | '~' | '>' | '<' | '*' | 'x' | 'X'))
}
