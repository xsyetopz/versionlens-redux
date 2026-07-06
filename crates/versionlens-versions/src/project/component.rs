use semver::Version;

use crate::model::ProjectVersionBump;
use crate::model::ProjectVersionBump::{Major, Minor, Patch};

type VersionBump = fn(&Version) -> Version;

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

pub(super) fn bump_version_component(version: Version, bump: ProjectVersionBump) -> Version {
    VERSION_COMPONENT_BUMPS
        .iter()
        .find_map(|entry| (entry.bump == bump).then(|| (entry.bumped)(&version)))
        .unwrap_or(version)
}

fn major_version_bump(version: &Version) -> Version {
    crate::semver_version(version.major + 1, 0, 0)
}

fn minor_version_bump(version: &Version) -> Version {
    crate::semver_version(version.major, version.minor + 1, 0)
}

fn patch_version_bump(version: &Version) -> Version {
    crate::semver_version(version.major, version.minor, version.patch + 1)
}
