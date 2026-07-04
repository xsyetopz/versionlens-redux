use crate::model::ProjectVersionBump;

use super::{is_prerelease_project_version, next_project_version};

#[test]
fn bumps_project_versions() {
    assert_eq!(
        next_project_version("1.2.3", None),
        Some("1.2.4".to_owned())
    );
    assert_eq!(
        next_project_version("1.2.3", Some(ProjectVersionBump::Major)),
        Some("2.0.0".to_owned())
    );
    assert_eq!(
        next_project_version("1.2.3", Some(ProjectVersionBump::Minor)),
        Some("1.3.0".to_owned())
    );
    assert_eq!(
        next_project_version("1.2.3-pre", None),
        Some("1.2.3".to_owned())
    );
    assert_eq!(
        next_project_version("1.2.3-pre", Some(ProjectVersionBump::Prerelease)),
        Some("1.2.3-pre.0".to_owned())
    );
}

#[test]
fn detects_prerelease_project_versions() {
    assert!(!is_prerelease_project_version("1.2.3"));
    assert!(is_prerelease_project_version("1.2.3-beta.4"));
}

#[test]
fn invalid_project_versions_fall_back_to_zero_without_coercion() {
    assert_eq!(
        next_project_version("release-1.2.3", Some(ProjectVersionBump::Major)),
        Some("1.0.0".to_owned())
    );
    assert_eq!(
        next_project_version("release-1.2.3", Some(ProjectVersionBump::Minor)),
        Some("0.1.0".to_owned())
    );
    assert_eq!(
        next_project_version("release-1.2.3", Some(ProjectVersionBump::Patch)),
        Some("0.0.1".to_owned())
    );
    assert!(!is_prerelease_project_version("release-1.2.3-beta.4"));
}
