use jsonc_parser::ast::Array as JsonArray;
use jsonc_parser::ast::StringLit;
use jsonc_parser::ast::Value::{Object as JsonValueObject, StringLit as JsonValueStringLit};

use crate::json_manifest::dependency::{
    JsonDependencyRanges, JsonDependencySource, collect_dependency_object,
    json_manifest_dependency, string_content_end, string_content_start,
};
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Npm;

use super::super::JsonManifestContext;

type JsonArrayPathDependencies = Vec<Dependency>;

pub(super) fn collect_json_array_path(
    context: &JsonManifestContext<'_>,
    path: &str,
    array: &JsonArray<'_>,
    out: &mut JsonArrayPathDependencies,
) {
    let source = JsonDependencySource {
        text: context.text,
        group: path,
        ecosystem: context.ecosystem,
    };
    for element in &array.elements {
        match element {
            JsonValueObject(object) => collect_dependency_object(&source, object, out),
            JsonValueStringLit(value) => collect_package_name_array_dependency(&source, value, out),
            _ => {}
        }
    }
}

fn collect_package_name_array_dependency(
    source: &JsonDependencySource<'_>,
    value: &StringLit<'_>,
    out: &mut JsonArrayPathDependencies,
) {
    if source.ecosystem != Npm || !is_npm_name_only_array_group(source.group) {
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
        "".to_owned(),
        JsonDependencyRanges {
            name_start,
            name_end,
            requirement_start: name_end,
            requirement_end: name_end,
        },
    ));
}

fn is_npm_name_only_array_group(group: &str) -> bool {
    matches!(
        group,
        "bundledDependencies" | "bundleDependencies" | "trustedDependencies"
    )
}
