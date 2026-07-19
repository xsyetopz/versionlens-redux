use crate::path_patterns::path_or_member_enabled_exact;
use versionlens_model::Dependency;

use super::super::paths::TomlPathContext;
use super::{TomlValueContext, push_keyed_dependency};

pub(super) fn collect_uv_source(
    context: &TomlValueContext<'_>,
    paths: &TomlPathContext,
    out: &mut Vec<Dependency>,
) -> bool {
    if paths.parent != "tool.uv.sources"
        || !path_or_member_enabled_exact(
            context.dependency_paths,
            "tool.uv.sources",
            context.keys.last().map(|key| key.get()),
        )
    {
        return false;
    }

    push_keyed_dependency(context, "tool.uv.sources", out);
    true
}
