use versionlens_parsers::{Dependency, Ecosystem};

pub(crate) fn is_npm_package_manager(dependency: &Dependency) -> bool {
    dependency.ecosystem == Ecosystem::Npm && dependency.group == "packageManager"
}
