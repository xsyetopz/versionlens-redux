use semver::Version;

use crate::model::ProjectVersionBump;

type VersionBump = fn(&Version) -> Version;

#[derive(Clone, Copy)]
struct VersionComponentBump {
    bump: ProjectVersionBump,
    bumped: VersionBump,
}

const VERSION_COMPONENT_BUMPS: &[VersionComponentBump] = &[
    VersionComponentBump {
        bump: ProjectVersionBump::Major,
        bumped: major_version_bump,
    },
    VersionComponentBump {
        bump: ProjectVersionBump::Minor,
        bumped: minor_version_bump,
    },
    VersionComponentBump {
        bump: ProjectVersionBump::Patch,
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
    Version::new(version.major + 1, 0, 0)
}

fn minor_version_bump(version: &Version) -> Version {
    Version::new(version.major, version.minor + 1, 0)
}

fn patch_version_bump(version: &Version) -> Version {
    Version::new(version.major, version.minor, version.patch + 1)
}
