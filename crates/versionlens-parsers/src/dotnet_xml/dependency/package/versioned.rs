use crate::model::Dependency;

use crate::dotnet_xml::{
    DotnetEventContext, DotnetTagKind,
    dependency::attrs::{DotnetDependencyAttrs, DotnetDependencyRange, dependency_from_attrs},
};

const VERSIONED_ATTRS: &[(&str, &str)] = &[
    ("Include", "VersionOverride"),
    ("Update", "VersionOverride"),
    ("Include", "Version"),
    ("Update", "Version"),
];

pub(super) fn versioned_package_dependency(
    context: &DotnetEventContext<'_>,
    group: &str,
    tag_kind: DotnetTagKind,
) -> Option<Dependency> {
    let range = match tag_kind {
        DotnetTagKind::Empty => DotnetDependencyRange::Tag,
        DotnetTagKind::Start => DotnetDependencyRange::Name,
    };

    VERSIONED_ATTRS
        .iter()
        .find_map(|(name_attr, version_attr)| {
            dependency_from_attrs(
                context,
                DotnetDependencyAttrs {
                    group,
                    name_attr,
                    version_attr,
                    range,
                },
            )
        })
}
