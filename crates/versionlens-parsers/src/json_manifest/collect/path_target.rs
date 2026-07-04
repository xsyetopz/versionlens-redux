use jsonc_parser::ast::Value;

use crate::json_manifest::dependency::{JsonDependencySource, collect_dependency_object};
use crate::model::Dependency;

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
    out: &mut Vec<Dependency>,
) {
    if let Value::Object(object) = context.target {
        let source = JsonDependencySource {
            text: context.manifest.text,
            group: context.path,
            ecosystem: context.manifest.ecosystem,
        };
        collect_dependency_object(&source, object, out);
        return;
    }

    if let Value::Array(array) = context.target {
        collect_json_array_path(context.manifest, context.path, array, out);
        return;
    }

    if let Value::StringLit(value) = context.target {
        collect_scalar_json_path(context, value, out);
    }
}
