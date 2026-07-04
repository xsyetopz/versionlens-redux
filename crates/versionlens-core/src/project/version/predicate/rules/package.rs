use versionlens_parsers::Dependency;

pub(super) fn is_cargo_package_version(dependency: &Dependency) -> bool {
    dependency.group == "package" && dependency.name == "version"
}

pub(super) fn is_maven_project_version(dependency: &Dependency) -> bool {
    dependency.group == "project.version" && dependency.name == "version"
}

pub(super) fn is_pub_version(dependency: &Dependency) -> bool {
    dependency.group == "version" && dependency.name == "version"
}
