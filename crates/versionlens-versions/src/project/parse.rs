use semver::Version;

use crate::parse::parse_version;

pub(super) fn project_version(raw: &str) -> Version {
    parse_version(raw).unwrap_or_else(|| Version::new(0, 0, 0))
}
