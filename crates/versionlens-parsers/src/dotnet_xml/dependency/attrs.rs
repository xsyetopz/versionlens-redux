use self::DotnetDependencyRange::Name as DotnetRangeName;
use self::DotnetDependencyRange::Tag as DotnetRangeTag;
use crate::positions::offset_range;
use versionlens_model::Dependency;

use super::super::DotnetEventContext;
use super::super::attributes::{attr_value, tag_bounds, version_insert};
use versionlens_model::Ecosystem::Dotnet;

pub(super) struct DotnetDependencyAttrs<'a> {
    pub(super) group: &'a str,
    pub(super) name_attr: &'a str,
    pub(super) version_attr: &'a str,
    pub(super) range: DotnetDependencyRange,
}

pub(super) struct DotnetMissingVersionAttrs<'a> {
    pub(super) group: &'a str,
    pub(super) name_attr: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DotnetDependencyRange {
    Name,
    Tag,
}

pub(super) fn dependency_from_attrs(
    context: &DotnetEventContext<'_>,
    attrs: DotnetDependencyAttrs<'_>,
) -> Option<Dependency> {
    let (tag_start, tag_end) = tag_bounds(context.text, context.span.start, context.span.end);
    let tag = context.text.get(tag_start..tag_end)?;
    let name = attr_value(tag, attrs.name_attr)?;
    let version = attr_value(tag, attrs.version_attr)?;
    let name_start = tag_start + name.range.start;
    let version_start = tag_start + version.range.start;
    let range = match attrs.range {
        DotnetRangeName => offset_range(context.text, name_start, name_start + name.len),
        DotnetRangeTag => offset_range(context.text, tag_start, tag_end),
    };

    Some(Dependency {
        name: name.value,
        requirement: version.value,
        ecosystem: Dotnet,
        group: attrs.group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range,
        requirement_range: offset_range(context.text, version_start, version_start + version.len),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

pub(super) fn missing_version_dependency(
    context: &DotnetEventContext<'_>,
    attrs: DotnetMissingVersionAttrs<'_>,
) -> Option<Dependency> {
    let (tag_start, tag_end) = tag_bounds(context.text, context.span.start, context.span.end);
    let tag = context.text.get(tag_start..tag_end)?;
    let name = attr_value(tag, attrs.name_attr)?;
    let name_start = tag_start + name.range.start;
    let (insert_offset, separator) = version_insert(tag)?;

    Some(Dependency {
        name: name.value,
        requirement: "*".to_owned(),
        ecosystem: Dotnet,
        group: attrs.group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(context.text, name_start, name_start + name.len),
        requirement_range: offset_range(
            context.text,
            tag_start + insert_offset,
            tag_start + insert_offset,
        ),
        requirement_prefix: format!("{separator}Version=\""),
        requirement_suffix: "\"".to_owned(),
    })
}
