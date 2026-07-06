use semver::Version;

pub(super) fn next_patch(version: &Version) -> Version {
    crate::semver_version(version.major, version.minor, version.patch + 1)
}
