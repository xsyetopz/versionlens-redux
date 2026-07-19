use semver::Version;

use crate::policy::ProjectVersionBump;
use crate::policy::ProjectVersionBump::{Major, Minor, Patch};

type VersionBump = fn(&Version) -> Option<Version>;

#[derive(Clone, Copy)]
struct VersionComponentBump {
    bump: ProjectVersionBump,
    bumped: VersionBump,
}

const VERSION_COMPONENT_BUMPS: &[VersionComponentBump] = &[
    VersionComponentBump {
        bump: Major,
        bumped: major_version_bump,
    },
    VersionComponentBump {
        bump: Minor,
        bumped: minor_version_bump,
    },
    VersionComponentBump {
        bump: Patch,
        bumped: patch_version_bump,
    },
];

pub(super) fn bump_version_component(
    version: Version,
    bump: ProjectVersionBump,
) -> Option<Version> {
    VERSION_COMPONENT_BUMPS
        .iter()
        .find_map(|entry| (entry.bump == bump).then(|| (entry.bumped)(&version)))
        .unwrap_or(Some(version))
}

fn major_version_bump(version: &Version) -> Option<Version> {
    Some(crate::semver_version(version.major.checked_add(1)?, 0, 0))
}

fn minor_version_bump(version: &Version) -> Option<Version> {
    Some(crate::semver_version(
        version.major,
        version.minor.checked_add(1)?,
        0,
    ))
}

fn patch_version_bump(version: &Version) -> Option<Version> {
    Some(crate::semver_version(
        version.major,
        version.minor,
        version.patch.checked_add(1)?,
    ))
}
