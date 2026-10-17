mod comparator;
mod nuget;
mod pessimistic;
mod semver_range;

pub use comparator::requirement_has_empty_comparator_intersection;
pub(super) use nuget::{nuget_requirement, nuget_requirement_satisfies};
pub(super) use pessimistic::pessimistic_update_available;
pub(super) use semver_range::semver_range_requirement_satisfies;

use pessimistic::pessimistic_requirement_satisfies;
use semver_range::looks_like_range;

pub(super) fn range_requirement_satisfies(requirement: &str, latest: &str) -> bool {
    if requirement.contains("||") {
        return disjunctive_requirement_satisfies(requirement, latest).unwrap_or(false);
    }

    simple_range_requirement_satisfies(requirement, latest)
}

pub(super) fn disjunctive_requirement_satisfies(requirement: &str, latest: &str) -> Option<bool> {
    if !requirement.contains("||") {
        return None;
    }

    Some(
        requirement
            .split("||")
            .map(|value| value.trim())
            .filter(|part| !part.is_empty())
            .any(|part| simple_range_requirement_satisfies(part, latest)),
    )
}

fn simple_range_requirement_satisfies(requirement: &str, latest: &str) -> bool {
    if let Some(satisfies) = pessimistic_requirement_satisfies(requirement, latest) {
        return satisfies;
    }
    if let Some(satisfies) = nuget_requirement_satisfies(requirement, latest) {
        return satisfies;
    }

    if !looks_like_range(requirement) {
        return false;
    }

    semver_range_requirement_satisfies(requirement, latest).unwrap_or(false)
}
