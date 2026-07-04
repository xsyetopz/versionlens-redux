use jsonc_parser::ast::{ObjectProp, StringLit};

use crate::json_manifest::npm;
use crate::model::Dependency;

use super::super::{
    JsonDependencyRanges, JsonDependencySource, json_manifest_dependency, property_name_range,
    string_content_start,
};
use super::literal::string_literal_json_manifest_dependency;

pub(super) fn npm_string_json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    prop: &ObjectProp<'_>,
    lit: &StringLit<'_>,
) -> Option<Dependency> {
    let value = lit.value.as_ref();
    if value.starts_with("catalog:") || value.starts_with("workspace:") {
        return None;
    }

    let value_start = string_content_start(lit.range.start, lit.range.end);
    if let Some(dependency) = npm::github_dependency(source, prop, value, value_start) {
        return Some(dependency);
    }
    if let Some(dependency) = npm_alias_json_manifest_dependency(source, prop, lit, value_start) {
        return Some(dependency);
    }
    if value.starts_with("npm:") {
        return None;
    }

    Some(string_literal_json_manifest_dependency(source, prop, lit))
}

fn npm_alias_json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    prop: &ObjectProp<'_>,
    lit: &StringLit<'_>,
    value_start: usize,
) -> Option<Dependency> {
    let (alias_name, alias_requirement, requirement_start) =
        npm::alias_dependency(lit.value.as_ref(), value_start)?;
    let (name_start, name_end) = property_name_range(prop);
    let requirement_end = if alias_requirement.is_empty() {
        requirement_start
    } else {
        value_start + lit.value.len()
    };
    let requirement_start = if alias_requirement.is_empty() {
        requirement_start
    } else {
        value_start
    };
    let mut dependency = json_manifest_dependency(
        source,
        alias_name,
        alias_requirement.to_owned(),
        JsonDependencyRanges {
            name_start,
            name_end,
            requirement_start,
            requirement_end,
        },
    );
    dependency.requirement_prefix = format!("npm:{alias_name}@");
    Some(dependency)
}
