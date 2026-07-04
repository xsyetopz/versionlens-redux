use semver::Version;

use crate::parse::{parse_padded_version, parse_version, strip_version_prefix};

pub(in crate::range) fn pessimistic_requirement_satisfies(
    requirement: &str,
    latest: &str,
) -> Option<bool> {
    let (lower, upper) = pessimistic_bounds(requirement)?;
    let latest = parse_version(latest)?;

    Some(latest >= lower && latest < upper)
}

pub(in crate::range) fn pessimistic_update_available(
    requirement: &str,
    latest: &str,
) -> Option<bool> {
    let (lower, upper) = pessimistic_bounds(requirement)?;
    let latest = parse_version(latest)?;

    Some(latest > lower && latest >= upper)
}

fn pessimistic_bounds(requirement: &str) -> Option<(Version, Version)> {
    let lower = requirement.trim().strip_prefix("~>")?.trim();
    let component_count = lower.split('.').count();
    let lower = parse_padded_version(&strip_version_prefix(lower))?;
    let upper = if component_count >= 3 {
        Version::new(lower.major, lower.minor + 1, 0)
    } else {
        Version::new(lower.major + 1, 0, 0)
    };

    Some((lower, upper))
}
