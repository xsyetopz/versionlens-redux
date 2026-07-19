use jsonc_parser::ast::Value::{
    Array as JsonValueArray, Object as JsonValueObject, StringLit as JsonValueStringLit,
};
use jsonc_parser::ast::{Object, Value};

use crate::json_manifest::dependency::{
    JsonDependencyRanges, JsonDependencySource, collect_dependency_object,
    json_manifest_dependency, string_content_end, string_content_start,
};
use versionlens_model::Dependency;

type JsonPathTargetDependencies = Vec<Dependency>;

use super::JsonManifestContext;

mod array;
mod scalar;

use array::collect_json_array_path;
use scalar::collect_scalar_json_path;

pub(super) struct JsonPathTargetContext<'a> {
    pub(super) manifest: &'a JsonManifestContext<'a>,
    pub(super) path: &'a str,
    pub(super) target: &'a Value<'a>,
    pub(super) parents: &'a [&'a str],
    pub(super) last: &'a str,
}

pub(super) fn collect_json_path_target(
    context: &JsonPathTargetContext<'_>,
    out: &mut JsonPathTargetDependencies,
) {
    if is_npm_name_only_array_group(context.path) && !matches!(context.target, JsonValueArray(_)) {
        return;
    }

    if let JsonValueObject(object) = context.target {
        if context.path == "devEngines.packageManager" {
            collect_dev_engines_package_manager(context, object, out);
            return;
        }

        let source = JsonDependencySource {
            text: context.manifest.text,
            group: context.path,
            ecosystem: context.manifest.ecosystem,
        };
        collect_dependency_object(&source, object, out);
        return;
    }

    if let JsonValueArray(array) = context.target {
        collect_json_array_path(context.manifest, context.path, array, out);
        return;
    }

    if let JsonValueStringLit(value) = context.target {
        collect_scalar_json_path(context, value, out);
    }
}

fn is_npm_name_only_array_group(path: &str) -> bool {
    matches!(
        path,
        "bundledDependencies" | "bundleDependencies" | "trustedDependencies"
    )
}

fn collect_dev_engines_package_manager(
    context: &JsonPathTargetContext<'_>,
    object: &Object<'_>,
    out: &mut JsonPathTargetDependencies,
) {
    let Some(name) = object.get_string("name") else {
        return;
    };
    let Some(version) = object.get_string("version") else {
        return;
    };
    let source = JsonDependencySource {
        text: context.manifest.text,
        group: context.path,
        ecosystem: context.manifest.ecosystem,
    };
    out.push(json_manifest_dependency(
        &source,
        name.value.as_ref(),
        version.value.as_ref().to_owned(),
        JsonDependencyRanges {
            name_start: string_content_start(name.range.start, name.range.end),
            name_end: string_content_end(name.range.start, name.range.end),
            requirement_start: string_content_start(version.range.start, version.range.end),
            requirement_end: string_content_end(version.range.start, version.range.end),
        },
    ));
}
