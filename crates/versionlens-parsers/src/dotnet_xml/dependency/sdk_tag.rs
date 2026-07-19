use self::DotnetDependencyRange::Name as DotnetRangeName;
use versionlens_model::Dependency;

use super::super::DotnetEventContext;
use super::attrs::{DotnetDependencyAttrs, DotnetDependencyRange, dependency_from_attrs};

pub(super) fn sdk_dependency_from_tag(context: &DotnetEventContext<'_>) -> Option<Dependency> {
    dependency_from_attrs(
        context,
        DotnetDependencyAttrs {
            group: "Sdk",
            name_attr: "Name",
            version_attr: "Version",
            range: DotnetRangeName,
        },
    )
}
