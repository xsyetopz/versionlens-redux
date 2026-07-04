use semver::Version;

use crate::model::ProjectVersionBump;

use super::component::bump_version_component;
use super::prerelease::{increment_prerelease, release_version};

pub(super) fn default_project_version_bump(version: &Version) -> ProjectVersionBump {
    if version.pre.is_empty() {
        return ProjectVersionBump::Patch;
    }

    ProjectVersionBump::Release
}

pub(super) fn bumped_project_version(
    version: Version,
    bump: ProjectVersionBump,
) -> Option<Version> {
    match bump {
        ProjectVersionBump::Major | ProjectVersionBump::Minor | ProjectVersionBump::Patch => {
            Some(bump_version_component(version, bump))
        }
        ProjectVersionBump::Release => Some(release_version(version)),
        ProjectVersionBump::Prerelease => increment_prerelease(version),
    }
}
