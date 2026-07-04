use jsonc_parser::ast::Value;

use crate::model::Dependency;

use super::JsonManifestContext;
use super::path::{object_at_path, value_at_path};
use crate::json_manifest::dependency::{JsonDependencySource, collect_dependency_object};

pub(super) fn collect_terminal_wildcard_path(
    context: &JsonManifestContext<'_>,
    parents: &[&str],
    out: &mut Vec<Dependency>,
) {
    let parent_path = parents.join(".");
    let Some(parent) = object_at_path(context.root, parents) else {
        return;
    };
    for prop in &parent.properties {
        if let Value::Object(child) = &prop.value {
            let group = terminal_wildcard_group(&parent_path, prop.name.as_str());
            let source = JsonDependencySource {
                text: context.text,
                group: &group,
                ecosystem: context.ecosystem,
            };
            collect_dependency_object(&source, child, out);
        }
    }
}

fn terminal_wildcard_group(parent_path: &str, child_name: &str) -> String {
    if parent_path == "workspaces.catalogs" {
        format!("{parent_path}.{child_name}")
    } else {
        parent_path.to_owned()
    }
}

pub(super) fn collect_json_wildcard_path(
    context: &JsonManifestContext<'_>,
    path: &[&str],
    star: usize,
    out: &mut Vec<Dependency>,
) {
    let Some(parent) = object_at_path(context.root, &path[..star]) else {
        return;
    };
    let child_path = &path[star + 1..];
    for prop in &parent.properties {
        let Value::Object(child) = &prop.value else {
            continue;
        };
        let Some(Value::Object(object)) = value_at_path(child, child_path) else {
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
