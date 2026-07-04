use jsonc_parser::ast::{ObjectProp, StringLit};

use crate::json_manifest::npm;
use crate::model::{Dependency, Ecosystem};

use super::super::{
    JsonDependencyRanges, JsonDependencySource, json_manifest_dependency, property_name_range,
    string_content_end, string_content_start,
};

pub(super) fn string_literal_json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    prop: &ObjectProp<'_>,
    lit: &StringLit<'_>,
) -> Dependency {
    let name = prop.name.as_str();
    let (name_start, name_end) = property_name_range(prop);
    json_manifest_dependency(
        source,
        dependency_selector(name, source.ecosystem),
        string_requirement(lit.value.as_ref(), source.ecosystem),
        JsonDependencyRanges {
            name_start,
            name_end,
            requirement_start: string_content_start(lit.range.start, lit.range.end),
            requirement_end: string_content_end(lit.range.start, lit.range.end),
        },
    )
}

fn string_requirement(value: &str, ecosystem: Ecosystem) -> String {
    if ecosystem == Ecosystem::Npm {
        return npm::string_requirement(value);
    }
    value.to_owned()
}

fn dependency_selector(name: &str, ecosystem: Ecosystem) -> &str {
    if ecosystem == Ecosystem::Npm {
        npm::trim_selector(name)
    } else {
        name
    }
}
