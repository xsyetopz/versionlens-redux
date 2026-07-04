mod bound;
mod patch;

use crate::parse::normalize_requirement;

use bound::{ComparatorBound, comparator_bound, max_lower_bound, min_upper_bound};
use patch::next_patch;

pub fn requirement_has_empty_comparator_intersection(requirement: &str) -> bool {
    let mut lower: Option<ComparatorBound> = None;
    let mut upper: Option<ComparatorBound> = None;

    for part in normalize_requirement(requirement)
        .split(|char: char| char.is_whitespace() || char == ',')
        .filter(|part| !part.is_empty())
    {
        if let Some((version, inclusive)) = comparator_bound(part, ">=", ">") {
            lower = Some(max_lower_bound(lower, version, inclusive));
        } else if let Some((version, inclusive)) = comparator_bound(part, "<=", "<") {
            upper = Some(min_upper_bound(upper, version, inclusive));
        }
    }

    let (Some((lower, lower_inclusive)), Some((upper, upper_inclusive))) = (lower, upper) else {
        return false;
    };

    lower > upper
        || (lower == upper && (!lower_inclusive || !upper_inclusive))
        || (!lower_inclusive && !upper_inclusive && next_patch(&lower) == upper)
}
