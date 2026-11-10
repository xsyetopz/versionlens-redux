use semver::Version;

use crate::versions::parse::parse_version;
use crate::versions::policy::ProjectVersionBump;

mod bump;
mod component;
mod prerelease;

use bump::{bumped_project_version, default_project_version_bump};

pub fn next_project_version(raw: &str, bump: Option<ProjectVersionBump>) -> Option<String> {
    let version = project_version(raw);
    let bump = bump.unwrap_or_else(|| default_project_version_bump(&version));
    bumped_project_version(version, bump).map(|next| next.to_string())
}

pub fn is_prerelease_project_version(raw: &str) -> bool {
    !project_version(raw).pre.is_empty()
}

fn project_version(raw: &str) -> Version {
    parse_version(raw).unwrap_or_else(|| crate::versions::semver_version(0, 0, 0))
}

#[cfg(test)]
mod tests {
use super::{is_prerelease_project_version, next_project_version};
use crate::versions::policy::ProjectVersionBump::{Major, Minor, Patch, Prerelease};

#[test]
fn bumps_project_versions() {
    assert_eq!(
        next_project_version("1.2.3", None),
        Some("1.2.4".to_owned())
    );
    assert_eq!(
        next_project_version("1.2.3", Some(Major)),
        Some("2.0.0".to_owned())
    );
    assert_eq!(
        next_project_version("1.2.3", Some(Minor)),
        Some("1.3.0".to_owned())
    );
    assert_eq!(
        next_project_version("1.2.3-pre", None),
        Some("1.2.3".to_owned())
    );
    assert_eq!(
        next_project_version("1.2.3-pre", Some(Prerelease)),
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
        next_project_version("release-1.2.3", Some(Major)),
        Some("1.0.0".to_owned())
    );
    assert_eq!(
        next_project_version("release-1.2.3", Some(Minor)),
        Some("0.1.0".to_owned())
    );
    assert_eq!(
        next_project_version("release-1.2.3", Some(Patch)),
        Some("0.0.1".to_owned())
    );
    assert!(!is_prerelease_project_version("release-1.2.3-beta.4"));
}

#[test]
fn project_version_bumps_do_not_overflow() {
    assert_eq!(
        next_project_version("18446744073709551615.0.0", Some(Major)),
        None
    );
    assert_eq!(
        next_project_version("1.18446744073709551615.0", Some(Minor)),
        None
    );
    assert_eq!(
        next_project_version("1.0.18446744073709551615", Some(Patch)),
        None
    );
    assert_eq!(
        next_project_version("1.0.0-rc.18446744073709551615", Some(Prerelease),),
        Some("1.0.0-rc.18446744073709551615.0".to_owned())
    );
}
}
