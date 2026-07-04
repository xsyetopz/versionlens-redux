use std::cmp::Ordering;

use crate::parse::parse_version;

pub fn compare_versions(left: &str, right: &str) -> Option<Ordering> {
    Some(parse_version(left)?.cmp(&parse_version(right)?))
}

pub fn is_newer(left: &str, right: &str) -> bool {
    matches!(compare_versions(left, right), Some(Ordering::Greater))
}
