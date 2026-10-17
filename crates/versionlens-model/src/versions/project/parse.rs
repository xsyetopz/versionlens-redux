use semver::Version;

use crate::versions::parse::parse_version;

pub(super) fn project_version(raw: &str) -> Version {
    parse_version(raw).unwrap_or_else(|| crate::versions::semver_version(0, 0, 0))
}
