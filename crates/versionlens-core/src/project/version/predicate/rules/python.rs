use versionlens_parsers::Dependency;

pub(super) fn is_python_project_version(dependency: &Dependency) -> bool {
    dependency.group == "project" && dependency.name == "version"
}
