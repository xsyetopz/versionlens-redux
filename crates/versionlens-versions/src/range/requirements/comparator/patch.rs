use semver::Version;

pub(super) fn next_patch(version: &Version) -> Version {
    Version::new(version.major, version.minor, version.patch + 1)
}
