use versionlens_model::{Dependency, VersionableKind};

pub(crate) fn is_project_version_dependency(dependency: &Dependency) -> bool {
    dependency.versionable_kind() == VersionableKind::ProjectVersion
}
