use crate::model::Ecosystem;
use jsonc_parser::ast::{Object, ObjectProp};

use crate::model::Dependency;

use super::{
    JsonDependencyRanges, JsonDependencySource, json_manifest_dependency, property_name_range,
    string_content_end, string_content_start,
};
use crate::model::Ecosystem::Dub;

pub(super) fn object_json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    prop: &ObjectProp<'_>,
    object: &Object<'_>,
) -> Option<Dependency> {
    let name = prop.name.as_str();
    let fields = object_dependency_fields(source.ecosystem);
    for field in fields {
        if let Some(value) = object.get_string(field) {
            let (name_start, name_end) = property_name_range(prop);
            let mut dependency = json_manifest_dependency(
                source,
                name,
                value.value.as_ref().to_owned(),
                JsonDependencyRanges {
                    name_start,
                    name_end,
                    requirement_start: string_content_start(value.range.start, value.range.end),
                    requirement_end: string_content_end(value.range.start, value.range.end),
                },
            );
            if *field == "path" {
                dependency.hosted_url = Some("path".to_owned());
            }
            return Some(dependency);
        }
    }
    None
}

fn object_dependency_fields(ecosystem: Ecosystem) -> &'static [&'static str] {
    if ecosystem == Dub {
        &["path", "repository", "version"]
    } else {
        &["version", "path", "repository"]
    }
}
