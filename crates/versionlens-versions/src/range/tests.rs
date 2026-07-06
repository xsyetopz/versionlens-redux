use std::cmp::Ordering::Less as OrderingLess;

use super::requirements::nuget_requirement;
use super::{
    build_variants, compare_versions, is_dotnet_requirement_parseable, is_newer,
    is_update_available, update_level,
};
use crate::model::UpdateLevel::{Major, Minor, Patch};

#[test]
fn compares_prefixed_versions() {
    assert_eq!(compare_versions("^1.2.3", "1.2.4"), Some(OrderingLess));
    assert_eq!(compare_versions("== 1.2.3", "1.2.4"), Some(OrderingLess));
    assert_eq!(compare_versions("==2.32", "2.33.0"), Some(OrderingLess));
    assert_eq!(compare_versions(">=1.2.3", "1.2.4"), Some(OrderingLess));
    assert_eq!(compare_versions("<= 1.2.3", "1.2.4"), Some(OrderingLess));
    assert!(is_newer("2.0.0", "~1.9.9"));
}

#[test]
fn range_satisfying_latest_is_not_an_update() {
    assert!(!is_update_available("1.2.0", "^1.0.0"));
    assert!(!is_update_available("1.18.3", "^v1.18.3"));
    assert!(!is_update_available("1.18.9", "~> v1.18.3"));
    assert!(!is_update_available("2.9.0", ">=2.0.0 <3.0.0"));
    assert!(!is_update_available("1.2.0", ">= 0.44.0 and < 2.0.0"));
    assert!(!is_update_available("2.9.0", ">=v2.0.0 <v3.0.0"));
    assert!(!is_update_available("3.0.0", "*"));
    assert!(!is_update_available("3.0.0", "any"));
    assert!(!is_update_available("3.0.0", "ANY"));
    assert!(!is_update_available("1.2.9", "~> 1.2.3"));
    assert!(!is_update_available("2.0.0", "[1.0.0,2.0.0]"));
    assert!(!is_update_available("1.9.9", "(1.0.0,2.0.0)"));
    assert!(!is_update_available("1.0.1", "(1.0.0,)"));
    assert!(!is_update_available("1.0.0", "(,1.0.0]"));
    assert!(!is_update_available("1.0.0", "[1.0.0]"));
    assert!(!is_update_available("2.0.0", "[1,2]"));
    assert!(!is_update_available(
        "3.4.5",
        ">=1.0.0 <2.0.0 || >=3.0.0 <4.0.0"
    ));
}

#[test]
fn detects_parseable_dotnet_requirements() {
    assert!(is_dotnet_requirement_parseable("1.0.0"));
    assert!(is_dotnet_requirement_parseable("1.0.0-beta"));
    assert!(is_dotnet_requirement_parseable("[1.0.0,2.0.0)"));
    assert!(is_dotnet_requirement_parseable("[1,2]"));
    assert!(!is_dotnet_requirement_parseable(""));
    assert!(!is_dotnet_requirement_parseable("invalid"));
    assert!(!is_dotnet_requirement_parseable("1.0.0.1"));
}

#[test]
fn converts_basic_nuget_ranges_to_semver_ranges() {
    assert_eq!(nuget_requirement("(1.0.0,)").as_deref(), Some(">1.0.0"));
    assert_eq!(nuget_requirement("[1.0.0]").as_deref(), Some("=1.0.0"));
    assert_eq!(nuget_requirement("(,1.0.0]").as_deref(), Some("<=1.0.0"));
    assert_eq!(
        nuget_requirement("[1.0.0,2.0.0]").as_deref(),
        Some(">=1.0.0, <=2.0.0")
    );
    assert_eq!(
        nuget_requirement("(1.0.0,2.0.0)").as_deref(),
        Some(">1.0.0, <2.0.0")
    );
    assert_eq!(
        nuget_requirement("[1.0.0,2.0.0)").as_deref(),
        Some(">=1.0.0, <2.0.0")
    );
}

#[test]
fn converts_partial_nuget_ranges_to_semver_ranges() {
    assert_eq!(
        nuget_requirement("[1,2]").as_deref(),
        Some(">=1.0.0, <=2.0.0")
    );
    assert_eq!(
        nuget_requirement("(1,2)").as_deref(),
        Some(">1.0.0, <2.0.0")
    );
}

#[test]
fn rejects_invalid_nuget_ranges() {
    assert_eq!(nuget_requirement("1."), None);
    assert_eq!(nuget_requirement("1.0."), None);
    assert_eq!(nuget_requirement("s.2.0"), None);
    assert_eq!(nuget_requirement("beta"), None);
}

#[test]
fn dotnet_floating_ranges_satisfy_matching_latest_versions() {
    assert!(!is_update_available("1.5.0", "1.*"));
    assert!(!is_update_available("1.0.9", "1.0.*"));
    assert!(is_update_available("2.0.0", "1.*"));
    assert!(is_update_available("1.1.0", "1.0.*"));
}

#[test]
fn empty_ranges_can_update_to_latest() {
    assert!(is_update_available("5.0.0", ">1 <1"));
    assert!(is_update_available("5.0.0", ">1.0.0 <1.0.1"));
    assert!(is_update_available("5.0.0", ">2 <1"));
}

#[test]
fn exact_pins_still_update() {
    assert!(is_update_available("1.0.0", ""));
    assert!(!is_update_available("1.0.0", "1"));
    assert!(is_update_available("1.2.4", "1.2.3"));
    assert!(is_update_available("2.0.0", "^1.9.9"));
    assert!(is_update_available("1.3.0", "~> 1.2.3"));
    assert!(is_update_available("3.0.0", ">=2.0.0 <3.0.0"));
    assert!(is_update_available("2.0.0", ">= 0.44.0 and < 2.0.0"));
    assert!(is_update_available("2.0.1", "[1.0.0,2.0.0]"));
    assert!(is_update_available("1.0.0", "(1.0.0,2.0.0)"));
    assert!(is_update_available("2.0.0", "[1.0.0,2.0.0)"));
    assert!(is_update_available("1.0.1", "[1.0.0]"));
    assert!(is_update_available("1.0.1", "(,1.0.0]"));
    assert!(is_update_available(
        "2.5.0",
        ">=1.0.0 <2.0.0 || >=3.0.0 <4.0.0"
    ));
}

#[test]
fn build_variants_match_upstream_coerced_version_family() {
    let versions = [
        "1.0.0+build.1",
        "release-1.0.0+build.2",
        "v1.0.0+build.3",
        "1.1.0+build.1",
    ];

    assert_eq!(
        build_variants("1.0.0+build.1", versions),
        [
            "1.0.0+build.1".to_owned(),
            "release-1.0.0+build.2".to_owned(),
            "v1.0.0+build.3".to_owned()
        ]
    );
}

#[test]
fn classifies_update_level() {
    assert_eq!(update_level("2.0.0", "^1.9.9"), Some(Major));
    assert_eq!(update_level("1.3.0", "~1.2.0"), Some(Minor));
    assert_eq!(update_level("1.3.0", ">=1.2.0"), Some(Minor));
    assert_eq!(update_level("3.0.0", ">=2.0.0 <3.0.0"), Some(Major));
    assert_eq!(update_level("1.2.4", "1.2.3"), Some(Patch));
    assert_eq!(update_level("1.2.3", "1.2.3"), None);
}
