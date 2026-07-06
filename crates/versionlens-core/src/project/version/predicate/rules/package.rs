use versionlens_parsers::Dependency;

pub(super) fn is_cargo_package_version(dependency: &Dependency) -> bool {
    dependency.group == "package" && dependency.name == "version"
}

pub(super) fn is_deno_project_version(dependency: &Dependency) -> bool {
    dependency.group == "version" && dependency.name.starts_with('@')
}

pub(super) fn is_hex_project_version(dependency: &Dependency) -> bool {
    dependency.group == "version" && !dependency.name.is_empty()
}

pub(super) fn is_maven_project_version(dependency: &Dependency) -> bool {
    dependency.group == "project.version" && dependency.name == "version"
}

pub(super) fn is_pub_version(dependency: &Dependency) -> bool {
    dependency.group == "version" && dependency.name == "version"
}
