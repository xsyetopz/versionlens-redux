use crate::json_manifest::npm::trim_package_descriptor;
use jsonc_parser::ast::ObjectProp as JsonObjectProp;
use jsonc_parser::ast::Value::{Array as JsonValueArray, Object as JsonValueObject};
use jsonc_parser::ast::{Object, Value};

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Npm;

type JsonManifestDependencies = Vec<Dependency>;
type JsonObject<'a> = &'a Object<'a>;

use super::JsonManifestContext;
use super::path::{object_at_path, value_at_path};
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

pub(super) fn collect_json_wildcard_path(
    context: &JsonManifestContext<'_>,
    path: &[&str],
    star: usize,
    out: &mut JsonManifestDependencies,
) {
    if collect_json_array_wildcard_path(context, path, star, out) {
        return;
    }

    let Some(parent) = object_at_path(context.root, &path[..star]) else {
        return;
    };
    let child_path = &path[star + 1..];
    for prop in &parent.properties {
        let JsonValueObject(child) = &prop.value else {
            continue;
        };
        let Some(JsonValueObject(object)) = value_at_path(child, child_path) else {
            continue;
        };
        let group = format!(
            "{}.{}.{}",
            path[..star].join("."),
            prop.name.as_str(),
            child_path.join(".")
        );
        let source = JsonDependencySource {
            text: context.text,
            group: &group,
            ecosystem: context.ecosystem,
        };
        collect_dependency_object(&source, object, out);
    }
}

fn collect_json_array_wildcard_path(
    context: &JsonManifestContext<'_>,
    path: &[&str],
    star: usize,
    out: &mut JsonManifestDependencies,
) -> bool {
    let Some(JsonValueArray(parent)) = value_at_path(context.root, &path[..star]) else {
        return false;
    };

    let child_path = &path[star + 1..];
    for (index, element) in parent.elements.iter().enumerate() {
        let JsonValueObject(child) = element else {
            continue;
        };
        let Some(JsonValueObject(object)) = value_at_path_from_object(child, child_path) else {
            continue;
        };
        let group = format!(
            "{}.{}.{}",
            path[..star].join("."),
            wildcard_array_segment(child, index),
            child_path.join(".")
        );
        let source = JsonDependencySource {
            text: context.text,
            group: &group,
            ecosystem: context.ecosystem,
        };
        collect_dependency_object(&source, object, out);
    }

    true
}

fn value_at_path_from_object<'a>(root: JsonObject<'a>, path: &[&str]) -> Option<&'a Value<'a>> {
    let mut current = root;
    let Some((last, parents)) = path.split_last() else {
        return None;
    };

    for segment in parents {
        current = current.get_object(segment)?;
    }
    current.get(last).map(|prop| &prop.value)
}

fn wildcard_array_segment(object: JsonObject<'_>, index: usize) -> String {
    object
        .get_string("name")
        .map(|name| name.value.to_string())
        .unwrap_or_else(|| index.to_string())
}
