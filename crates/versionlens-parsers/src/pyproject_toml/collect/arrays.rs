use crate::path_patterns::path_or_member_enabled_exact;
use versionlens_model::Dependency;

use super::super::dependencies::collect_requirement_array;
use super::super::paths::TomlPathContext;
use super::TomlValueContext;

pub(super) fn collect_array_dependency_group(
    context: &TomlValueContext<'_>,
    paths: &TomlPathContext,
    out: &mut Vec<Dependency>,
) -> bool {
    if !array_dependency_group_enabled(context, paths) {
        return false;
    }

    collect_requirement_array(context.text, &paths.path, context.value, out);
    true
}

fn array_dependency_group_enabled(context: &TomlValueContext<'_>, paths: &TomlPathContext) -> bool {
    let supported_path = paths.path == "project.dependencies"
        || paths.parent == "project.optional-dependencies"
        || paths.parent == "dependency-groups";
    supported_path
        && path_or_member_enabled_exact(
            context.dependency_paths,
            &paths.parent,
            context.keys.last().map(|key| key.get()),
        )
}
