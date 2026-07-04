use crate::model::Dependency;
use crate::path_patterns::path_or_member_enabled;

use super::super::paths::TomlPathContext;
use super::{TomlKind, TomlValueContext, push_keyed_dependency};

pub(super) fn collect_pipfile_dependency(
    context: &TomlValueContext<'_>,
    paths: &TomlPathContext,
    out: &mut Vec<Dependency>,
) -> bool {
    if context.kind != TomlKind::Pipfile
        || !matches!(paths.parent.as_str(), "packages" | "dev-packages")
        || !path_or_member_enabled(
            context.dependency_paths,
            &paths.parent,
            context.keys.last().map(|key| key.get()),
        )
    {
        return false;
    }

    push_keyed_dependency(context, &paths.parent, out);
    true
}
