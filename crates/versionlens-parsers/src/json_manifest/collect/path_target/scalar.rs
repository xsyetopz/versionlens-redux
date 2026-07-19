use jsonc_parser::ast::StringLit;

use crate::json_manifest::dependency::{
    JsonDependencySource, json_manifest_dependency_from_property, scalar_json_manifest_dependency,
};
use versionlens_model::Dependency;

type JsonPathTargetDependencies = Vec<Dependency>;
type JsonStringLit<'a> = &'a StringLit<'a>;

use super::super::path::object_at_path;
use super::JsonPathTargetContext;

pub(super) fn collect_scalar_json_path(
    context: &JsonPathTargetContext<'_>,
    value: JsonStringLit<'_>,
    out: &mut JsonPathTargetDependencies,
) {
    if context.parents.is_empty() {
        collect_root_scalar_json_path(context, value, out);
    } else {
        collect_nested_scalar_json_path(context, out);
    }
}

fn collect_root_scalar_json_path(
    context: &JsonPathTargetContext<'_>,
    value: JsonStringLit<'_>,
    out: &mut JsonPathTargetDependencies,
) {
    let manifest = context.manifest;
    let source = JsonDependencySource {
        text: manifest.text,
        group: context.path,
        ecosystem: manifest.ecosystem,
    };
    if let Some(prop) = manifest.root.get(context.path)
        && let Some(dependency) = scalar_json_manifest_dependency(&source, prop, value)
    {
        out.push(dependency);
    }
}

fn collect_nested_scalar_json_path(
    context: &JsonPathTargetContext<'_>,
    out: &mut JsonPathTargetDependencies,
) {
    let group = context.parents.join(".");
    let source = JsonDependencySource {
        text: context.manifest.text,
        group: &group,
        ecosystem: context.manifest.ecosystem,
    };
    if let Some(parent) = object_at_path(context.manifest.root, context.parents)
        && let Some(prop) = parent.get(context.last)
        && let Some(dependency) = json_manifest_dependency_from_property(&source, prop)
    {
        out.push(dependency);
    }
}
