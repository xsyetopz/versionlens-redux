mod missing;
mod tag;
mod versioned;

use crate::model::Dependency;

use super::super::{DotnetEventContext, DotnetTagKind};
use missing::missing_version_package_dependency;
pub(in crate::dotnet_xml) use tag::is_package_tag;
use versioned::versioned_package_dependency;

pub(super) fn package_dependencies_from_tag(
    context: &DotnetEventContext<'_>,
    group: &str,
    tag_kind: DotnetTagKind,
) -> Vec<Dependency> {
    package_dependency(context, group, tag_kind)
        .into_iter()
        .collect()
}

fn package_dependency(
    context: &DotnetEventContext<'_>,
    group: &str,
    tag_kind: DotnetTagKind,
) -> Option<Dependency> {
    versioned_package_dependency(context, group, tag_kind)
        .or_else(|| missing_version_package_dependency(context, group))
}
