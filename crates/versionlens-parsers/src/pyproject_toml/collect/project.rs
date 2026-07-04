use crate::model::Dependency;
use crate::path_patterns::path_or_member_enabled_exact;

use super::super::dependencies::{PythonKeyedDependencyInput, keyed_dependency};
use super::super::paths::TomlPathContext;
use super::TomlValueContext;

pub(super) fn collect_project_version(
    context: &TomlValueContext<'_>,
    paths: &TomlPathContext,
    out: &mut Vec<Dependency>,
) -> bool {
    if paths.path != "project.version"
        || !path_or_member_enabled_exact(context.dependency_paths, "project", None)
    {
        return false;
    }

    if let Some(version_key) = context.keys.get(1)
        && let Some(dependency) = keyed_dependency(PythonKeyedDependencyInput {
            text: context.text,
            group: "project",
            name: "version",
            value: context.value,
            key: version_key,
        })
    {
        out.push(dependency);
    }
    true
}
