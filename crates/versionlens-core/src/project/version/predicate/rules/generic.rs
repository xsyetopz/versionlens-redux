use versionlens_model::Dependency;

pub(super) fn is_name_version_requirement(dependency: &Dependency) -> bool {
    dependency.group == "version" && dependency.name == dependency.requirement
}
