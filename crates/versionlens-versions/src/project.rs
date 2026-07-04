use crate::model::ProjectVersionBump;

mod bump;
mod component;
mod parse;
mod prerelease;

use bump::{bumped_project_version, default_project_version_bump};
use parse::project_version;

pub fn next_project_version(raw: &str, bump: Option<ProjectVersionBump>) -> Option<String> {
    let version = project_version(raw);
    let bump = bump.unwrap_or_else(|| default_project_version_bump(&version));
    bumped_project_version(version, bump).map(|next| next.to_string())
}

pub fn is_prerelease_project_version(raw: &str) -> bool {
    !project_version(raw).pre.is_empty()
}

#[cfg(test)]
mod tests;
