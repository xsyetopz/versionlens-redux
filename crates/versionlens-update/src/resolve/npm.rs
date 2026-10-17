use versionlens_model::Ecosystem::Npm;
use versionlens_model::{Dependency, is_npm_dist_tag_requirement};

pub(super) fn is_npm_dist_tag_dependency(dependency: &Dependency, latest: &str) -> bool {
    let requirement = dependency.requirement.trim();
    dependency.ecosystem == Npm
        && requirement != "latest"
        && requirement != latest
        && is_npm_dist_tag_requirement(requirement)
}
