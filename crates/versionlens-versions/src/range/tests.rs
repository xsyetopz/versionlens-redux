use std::cmp::Ordering::Less as OrderingLess;

use super::requirements::nuget_requirement;
use super::{
    build_variants, compare_versions, compare_versions_for_dialect,
    is_dotnet_requirement_parseable, is_newer, is_update_available,
    is_update_available_for_dialect, requirement_is_parseable_for_dialect,
    requirement_satisfies_latest_for_dialect, update_level,
};
use crate::VersionDialect;
use crate::policy::UpdateLevel::{Major, Minor, Patch};

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

#[test]
fn older_provider_releases_do_not_create_downgrade_updates() {
    for requirement in ["2.0.0", ">=2.0.0", "^2.0.0", ">=2.0.0 <3.0.0"] {
        assert!(!is_update_available("1.9.9", requirement));
    }
}

#[test]
fn pep440_constraints_support_exclusions_and_extended_versions() {
    let dialect = VersionDialect::Pep440;
    for requirement in [
        ">=1!2.0.0.0,!=1!2.1.0.0",
        "~=1.4.5",
        "==1.0.*",
        ">=1.0rc1,<2.0",
    ] {
        assert!(requirement_is_parseable_for_dialect(
            requirement,
            "1!2.0.0.1",
            dialect,
        ));
    }
    assert!(requirement_satisfies_latest_for_dialect(
        ">=1!2.0.0.0,!=1!2.1.0.0",
        "1!2.0.0.4+linux.1",
        dialect,
    ));
    assert!(!requirement_satisfies_latest_for_dialect(
        ">=1.0,!=1.2.post1",
        "1.2.post1",
        dialect,
    ));
    assert_eq!(
        compare_versions_for_dialect("1.0.dev1", "1.0rc1", dialect),
        Some(OrderingLess)
    );
}

#[test]
fn pep440_update_checks_reject_downgrades_below_exact_or_lower_bounds() {
    let dialect = VersionDialect::Pep440;
    for requirement in ["==2.0", "2.0", ">=2.0,<3.0", "~=2.0"] {
        assert!(!is_update_available_for_dialect(
            "1.9.post1",
            requirement,
            dialect,
        ));
    }
    assert!(is_update_available_for_dialect(
        "2.1.post1",
        "==2.0",
        dialect,
    ));
}

#[test]
fn pep440_ordered_and_compatible_specifiers_ignore_candidate_local_labels() {
    let dialect = VersionDialect::Pep440;
    for (requirement, candidate, expected) in [
        ("<=1.0", "1.0+local", true),
        (">=1.0", "1.0+local", true),
        ("<1.0", "1.0+local", false),
        (">1.0", "1.0+local", false),
        ("~=1.0", "1.0+local", true),
        ("~=1.0", "2.0+local", false),
    ] {
        assert_eq!(
            requirement_satisfies_latest_for_dialect(requirement, candidate, dialect),
            expected,
            "{requirement} against {candidate}",
        );
    }

    assert!(!is_update_available_for_dialect(
        "1.0+local",
        "<=1.0",
        dialect,
    ));
}

#[test]
fn pep440_rejects_local_operands_where_the_spec_prohibits_them() {
    let dialect = VersionDialect::Pep440;
    for requirement in [
        "~=1.0+local",
        "<1.0+local",
        "<=1.0+local",
        ">1.0+local",
        ">=1.0+local",
    ] {
        assert!(!requirement_is_parseable_for_dialect(
            requirement,
            "1.0+local",
            dialect,
        ));
    }

    for requirement in ["==1.0+local", "!=1.0+local", "===1.0+local"] {
        assert!(requirement_is_parseable_for_dialect(
            requirement,
            "1.0+local",
            dialect,
        ));
    }
}
