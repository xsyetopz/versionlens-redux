use self::DotnetDependencyRange::Name as DotnetRangeName;
use self::DotnetTagKind::{Empty as DotnetTagEmpty, Start as DotnetTagStart};
use versionlens_model::Dependency;

use crate::dotnet_xml::dependency::attrs::DotnetDependencyRange::Tag as DotnetRangeTag;
use crate::dotnet_xml::dependency::attrs::{
    DotnetDependencyAttrs, DotnetDependencyRange, dependency_from_attrs,
};
use crate::dotnet_xml::{DotnetEventContext, DotnetTagKind};

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
        DotnetTagEmpty => DotnetRangeTag,
        DotnetTagStart => DotnetRangeName,
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
