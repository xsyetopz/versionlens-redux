use jsonc_parser::ast::{ObjectProp, StringLit};

use crate::model::{Dependency, Ecosystem};

use super::super::npm;
use super::{
    JsonDependencyRanges, JsonDependencySource, json_manifest_dependency, property_name_range,
    string_content_end, string_content_start,
};

pub(super) fn scalar_json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    prop: &ObjectProp<'_>,
    value: &StringLit<'_>,
) -> Option<Dependency> {
    let raw = value.value.as_ref();
    let value_start = string_content_start(value.range.start, value.range.end);
    let value_end = string_content_end(value.range.start, value.range.end);
    let parts = scalar_dependency_parts(source, raw, value_start, value_end)?;
    let (name_start, name_end) = property_name_range(prop);

    Some(json_manifest_dependency(
        source,
        parts.name,
        parts.requirement.to_owned(),
        JsonDependencyRanges {
            name_start,
            name_end,
            requirement_start: parts.requirement_start,
            requirement_end: parts.requirement_end,
        },
    ))
}

struct ScalarDependencyParts<'a> {
    name: &'a str,
    requirement: &'a str,
    requirement_start: usize,
    requirement_end: usize,
}

fn scalar_dependency_parts<'a>(
    source: &JsonDependencySource<'_>,
    raw: &'a str,
    value_start: usize,
    value_end: usize,
) -> Option<ScalarDependencyParts<'a>> {
    match source.group {
        "packageManager" if source.ecosystem == Ecosystem::Npm => {
            package_manager_parts(raw, value_end)
        }
        "version" => Some(ScalarDependencyParts {
            name: raw,
            requirement: raw,
            requirement_start: value_start,
            requirement_end: value_end,
        }),
        _ => None,
    }
}

fn package_manager_parts(raw: &str, value_end: usize) -> Option<ScalarDependencyParts<'_>> {
    let (name, requirement) = npm::parse_package_manager(raw)?;

    Some(ScalarDependencyParts {
        name,
        requirement,
        requirement_start: value_end.saturating_sub(requirement.len()),
        requirement_end: value_end,
    })
}
