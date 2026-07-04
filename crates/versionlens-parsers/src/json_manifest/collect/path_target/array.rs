use jsonc_parser::ast::{StringLit, Value};

use crate::json_manifest::dependency::{
    JsonDependencyRanges, JsonDependencySource, collect_dependency_object,
    json_manifest_dependency, string_content_end, string_content_start,
};
use crate::model::{Dependency, Ecosystem};

use super::super::JsonManifestContext;

pub(super) fn collect_json_array_path(
    context: &JsonManifestContext<'_>,
    path: &str,
    array: &jsonc_parser::ast::Array<'_>,
    out: &mut Vec<Dependency>,
) {
    let source = JsonDependencySource {
        text: context.text,
        group: path,
        ecosystem: context.ecosystem,
    };
    for element in &array.elements {
        match element {
            Value::Object(object) => collect_dependency_object(&source, object, out),
            Value::StringLit(value) => collect_package_name_array_dependency(&source, value, out),
            _ => {}
        }
    }
}

fn collect_package_name_array_dependency(
    source: &JsonDependencySource<'_>,
    value: &StringLit<'_>,
    out: &mut Vec<Dependency>,
) {
    if source.ecosystem != Ecosystem::Npm || !is_npm_bundle_group(source.group) {
        return;
    }

    let name = value.value.as_ref();
    if name.is_empty() {
        return;
    }

    let name_start = string_content_start(value.range.start, value.range.end);
    let name_end = string_content_end(value.range.start, value.range.end);
    out.push(json_manifest_dependency(
        source,
        name,
        String::new(),
        JsonDependencyRanges {
            name_start,
            name_end,
            requirement_start: name_end,
            requirement_end: name_end,
        },
    ));
}

fn is_npm_bundle_group(group: &str) -> bool {
    matches!(group, "bundledDependencies" | "bundleDependencies")
}
