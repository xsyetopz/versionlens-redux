use crate::json_manifest::npm::trim_package_descriptor;
use crate::manifest::path_query::query;
use jsonc_parser::ast::Object;
use jsonc_parser::ast::ObjectProp as JsonObjectProp;
use jsonc_parser::ast::Value::Object as JsonValueObject;

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Npm;

type JsonManifestDependencies = Vec<Dependency>;
type JsonObject<'a> = &'a Object<'a>;

use super::JsonManifestContext;
use super::node::JsonNode;
use super::path::object_at_path;
use crate::json_manifest::dependency::{
    JsonDependencyRanges, JsonDependencySource, collect_dependency_object,
    json_manifest_dependency, property_name_range, string_content_end, string_content_start,
};

pub(super) fn collect_terminal_wildcard_path(
    context: &JsonManifestContext<'_>,
    parents: &[&str],
    out: &mut JsonManifestDependencies,
) {
    let parent_path = parents.join(".");
    let Some(parent) = object_at_path(context.root, parents) else {
        return;
    };
    for prop in &parent.properties {
        if let JsonValueObject(child) = &prop.value {
            let group = terminal_wildcard_group(&parent_path, prop.name.as_str());
            let source = JsonDependencySource {
                text: context.text,
                group: &group,
                ecosystem: context.ecosystem,
            };
            collect_npm_override_self_dependency(context, &parent_path, prop, child, out);
            collect_dependency_object(&source, child, out);
        }
    }
}

fn collect_npm_override_self_dependency(
    context: &JsonManifestContext<'_>,
    parent_path: &str,
    prop: &JsonObjectProp<'_>,
    child: JsonObject<'_>,
    out: &mut JsonManifestDependencies,
) {
    if context.ecosystem != Npm || !matches!(parent_path, "overrides" | "pnpm.overrides") {
        return;
    }
    let Some(version) = child.get_string(".") else {
        return;
    };
    let (name_start, name_end) = property_name_range(prop);
    let source = JsonDependencySource {
        text: context.text,
        group: parent_path,
        ecosystem: context.ecosystem,
    };
    out.push(json_manifest_dependency(
        &source,
        trim_package_descriptor(prop.name.as_str()),
        version.value.as_ref().to_owned(),
        JsonDependencyRanges {
            name_start,
            name_end,
            requirement_start: string_content_start(version.range.start, version.range.end),
            requirement_end: string_content_end(version.range.start, version.range.end),
        },
    ));
}

fn terminal_wildcard_group(parent_path: &str, child_name: &str) -> String {
    if parent_path == "workspaces.catalogs" || parent_path == "catalogs" {
        format!("{parent_path}.{child_name}")
    } else {
        parent_path.to_owned()
    }
}

/// Collects every object that a path with a `*` segment reaches. The group
/// is the matched keys, so an array element is named by its `name` or index.
pub(super) fn collect_json_wildcard_path(
    context: &JsonManifestContext<'_>,
    segments: &[&str],
    out: &mut JsonManifestDependencies,
) {
    for found in query(JsonNode::Object(context.root), segments) {
        let JsonNode::Object(object) = found.node else {
            continue;
        };
        let group = found.keys.join(".");
        let source = JsonDependencySource {
            text: context.text,
            group: &group,
            ecosystem: context.ecosystem,
        };
        collect_dependency_object(&source, object, out);
    }
}
