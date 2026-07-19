use std::cmp::Ordering;
use std::cmp::Ordering::Greater as OrderingGreater;

use crate::VersionDialect;
use crate::parse::parse_version;
use crate::pep440;

pub fn compare_versions(left: &str, right: &str) -> Option<Ordering> {
    Some(parse_version(left)?.cmp(&parse_version(right)?))
}

pub fn compare_versions_for_dialect(
    left: &str,
    right: &str,
    dialect: VersionDialect,
) -> Option<Ordering> {
    match dialect {
        VersionDialect::Semver => compare_versions(left, right),
        VersionDialect::Pep440 => {
            Some(pep440::parse_version(left)?.cmp(&pep440::parse_version(right)?))
        }
    }
}

pub fn is_newer(left: &str, right: &str) -> bool {
    matches!(compare_versions(left, right), Some(OrderingGreater))
}
