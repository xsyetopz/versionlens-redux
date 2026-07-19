use std::collections::HashMap;

use versionlens_model::Dependency;

use super::nodes::XmlNode;

mod package;
mod project;
mod properties;

use package::package_dependency;
use project::project_version_dependency;
use properties::{collect_properties, resolve_property};

struct MavenParseContext<'a> {
    text: &'a str,
    nodes: &'a [XmlNode],
    properties: &'a HashMap<String, usize>,
    dependency_paths: &'a [&'a str],
}

pub(super) fn collect_maven_dependencies(
    text: &str,
    nodes: &[XmlNode],
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    let properties = collect_properties(nodes);
    let context = MavenParseContext {
        text,
        nodes,
        properties: &properties,
        dependency_paths,
    };

    nodes
        .iter()
        .filter_map(|node| maven_dependency_from_node(&context, node))
        .collect()
}

fn maven_dependency_from_node(
    context: &MavenParseContext<'_>,
    node: &XmlNode,
) -> Option<Dependency> {
    context
        .dependency_paths
        .contains(&node.path.as_str())
        .then(|| enabled_maven_dependency(context, node))
        .flatten()
}

fn enabled_maven_dependency(context: &MavenParseContext<'_>, node: &XmlNode) -> Option<Dependency> {
    if node.name == "version" {
        Some(project_version_dependency(context.text, node))
    } else {
        package_dependency(context, node)
    }
}
