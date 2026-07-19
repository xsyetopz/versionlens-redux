use self::object::object_json_manifest_dependency;
use self::scalar::scalar_json_manifest_dependency as scalar_json_manifest_dependency_impl;
use self::string::string_json_manifest_dependency;
use jsonc_parser::ast::Value::{Object as JsonValueObject, StringLit as JsonValueStringLit};
use jsonc_parser::ast::{Object, ObjectProp, StringLit};
use versionlens_model::Ecosystem;

use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::{Composer, Npm};

mod object;
mod ranges;
mod scalar;
mod string;

pub(super) use ranges::{property_name_range, string_content_end, string_content_start};

type JsonObjectProp<'a> = &'a ObjectProp<'a>;
type JsonStringLit<'a> = &'a StringLit<'a>;

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
    prop: JsonObjectProp<'_>,
) -> Option<Dependency> {
    if source.ecosystem == Npm
        && matches!(source.group, "overrides" | "pnpm.overrides")
        && prop.name.as_str() == "."
    {
        return None;
    }

    match &prop.value {
        JsonValueStringLit(lit) => string_json_manifest_dependency(source, prop, lit),
        JsonValueObject(object) => object_json_manifest_dependency(source, prop, object),
        _ => None,
    }
}

pub(super) fn scalar_json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    prop: JsonObjectProp<'_>,
    value: JsonStringLit<'_>,
) -> Option<Dependency> {
    scalar_json_manifest_dependency_impl(source, prop, value)
}

pub(super) fn json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    name: &str,
    requirement: String,
    ranges: JsonDependencyRanges,
) -> Dependency {
    let mut dependency = Dependency {
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
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    };
    apply_composer_requirement_suffix(&mut dependency);
    dependency
}

fn apply_composer_requirement_suffix(dependency: &mut Dependency) {
    if dependency.ecosystem != Composer {
        return;
    }

    let Some(suffix_start) = composer_suffix_start(&dependency.requirement) else {
        return;
    };
    dependency.requirement_suffix = dependency.requirement[suffix_start..].to_owned();
    dependency.requirement.truncate(suffix_start);
}

fn composer_suffix_start(requirement: &str) -> Option<usize> {
    if requirement.contains("://") || requirement.starts_with("git@") {
        return None;
    }

    if let Some(index) = requirement.find(" as ")
        && index > 0
    {
        return Some(index);
    }

    if let Some(index) = requirement.find('#')
        && index > 0
    {
        return Some(index);
    }

    let index = requirement.rfind('@')?;
    (index > 0).then_some(index)
}
