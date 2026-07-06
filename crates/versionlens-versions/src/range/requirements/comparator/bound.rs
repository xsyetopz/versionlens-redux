use semver::Version;

use crate::parse::parse_version;

pub(super) type ComparatorBound = (Version, bool);
type OptionalComparatorBound = Option<ComparatorBound>;

pub(super) fn comparator_bound(
    part: &str,
    inclusive_prefix: &str,
    exclusive_prefix: &str,
) -> OptionalComparatorBound {
    if let Some(version) = part.strip_prefix(inclusive_prefix) {
        return Some((parse_version(version)?, true));
    }
    Some((parse_version(part.strip_prefix(exclusive_prefix)?)?, false))
}

pub(super) fn max_lower_bound(
    current: OptionalComparatorBound,
    next: Version,
    inclusive: bool,
) -> ComparatorBound {
    match current {
        Some((current, current_inclusive))
            if current > next || (current == next && !current_inclusive) =>
        {
            (current, current_inclusive)
        }
        _ => (next, inclusive),
    }
}

pub(super) fn min_upper_bound(
    current: OptionalComparatorBound,
    next: Version,
    inclusive: bool,
) -> ComparatorBound {
    match current {
        Some((current, current_inclusive))
            if current < next || (current == next && !current_inclusive) =>
        {
            (current, current_inclusive)
        }
        _ => (next, inclusive),
    }
}
