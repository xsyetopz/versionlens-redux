use versionlens_model::Dependency;

mod rules;

use rules::project_version_match;

pub(crate) fn is_project_version_dependency(dependency: &Dependency) -> bool {
    project_version_match(dependency)
}
