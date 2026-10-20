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
mod tests;
