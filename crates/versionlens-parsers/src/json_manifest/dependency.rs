use jsonc_parser::ast::{Object, ObjectProp, Value};

use crate::model::{Dependency, Ecosystem};
use crate::positions::offset_range;

mod object;
mod ranges;
mod scalar;
mod string;

pub(super) use ranges::{property_name_range, string_content_end, string_content_start};

pub(super) struct JsonDependencySource<'a> {
    pub(super) text: &'a str,
    pub(super) group: &'a str,
    pub(super) ecosystem: Ecosystem,
}

pub(super) struct JsonDependencyRanges {
    pub(super) name_start: usize,
    pub(super) name_end: usize,
    pub(super) requirement_start: usize,
    pub(super) requirement_end: usize,
}

pub(super) fn collect_dependency_object(
    source: &JsonDependencySource<'_>,
    object: &Object<'_>,
    out: &mut Vec<Dependency>,
) {
    for prop in &object.properties {
        if let Some(dependency) = json_manifest_dependency_from_property(source, prop) {
            out.push(dependency);
        }
    }
}

pub(super) fn json_manifest_dependency_from_property(
    source: &JsonDependencySource<'_>,
    prop: &ObjectProp<'_>,
) -> Option<Dependency> {
    match &prop.value {
        Value::StringLit(lit) => string::string_json_manifest_dependency(source, prop, lit),
        Value::Object(object) => object::object_json_manifest_dependency(source, prop, object),
        _ => None,
    }
}

pub(super) fn scalar_json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    prop: &ObjectProp<'_>,
    value: &jsonc_parser::ast::StringLit<'_>,
) -> Option<Dependency> {
    scalar::scalar_json_manifest_dependency(source, prop, value)
}

pub(super) fn json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    name: &str,
    requirement: String,
    ranges: JsonDependencyRanges,
) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement,
        ecosystem: source.ecosystem,
        group: source.group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(source.text, ranges.name_start, ranges.name_end),
        requirement_range: offset_range(
            source.text,
            ranges.requirement_start,
            ranges.requirement_end,
        ),
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
    }
}
