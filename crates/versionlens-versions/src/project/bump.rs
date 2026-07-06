use semver::Version;

use crate::model::ProjectVersionBump;

use super::component::bump_version_component;
use super::prerelease::{increment_prerelease, release_version};
use crate::model::ProjectVersionBump::{Major, Minor, Patch, Prerelease, Release};

pub(super) fn default_project_version_bump(version: &Version) -> ProjectVersionBump {
    if version.pre.is_empty() {
        return Patch;
    }

    Release
}

pub(super) fn bumped_project_version(
    version: Version,
    bump: ProjectVersionBump,
) -> Option<Version> {
    match bump {
        Major | Minor | Patch => Some(bump_version_component(version, bump)),
        Release => Some(release_version(version)),
        Prerelease => increment_prerelease(version),
    }
}
