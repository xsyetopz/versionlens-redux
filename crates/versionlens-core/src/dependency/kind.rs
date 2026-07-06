use versionlens_parsers::Dependency;
use versionlens_parsers::Ecosystem::Npm;

pub(crate) fn is_npm_package_manager(dependency: &Dependency) -> bool {
    dependency.ecosystem == Npm
        && matches!(
            dependency.group.as_str(),
            "packageManager" | "devEngines.packageManager"
        )
}
