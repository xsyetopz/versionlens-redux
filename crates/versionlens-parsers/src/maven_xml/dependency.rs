use std::collections::HashMap;

use versionlens_model::Dependency;

use crate::manifest::adapter::xml::{XmlElement, XmlNode};
use crate::manifest::path_query::query;

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
    };

    selected_node_indexes(nodes, dependency_paths)
        .into_iter()
        .filter_map(|index| enabled_maven_dependency(&context, &nodes[index]))
        .collect()
}

/// The nodes that the dependency paths select, in the order of the nodes.
fn selected_node_indexes(nodes: &[XmlNode], dependency_paths: &[&str]) -> Vec<usize> {
    let mut indexes = dependency_paths
        .iter()
        .flat_map(|path| {
            query(
                XmlElement::root(nodes),
                &path.split('.').collect::<Vec<_>>(),
            )
        })
        .filter_map(|found| found.node.index())
        .collect::<Vec<_>>();
    indexes.sort_unstable();
    indexes.dedup();
    indexes
}

fn enabled_maven_dependency(context: &MavenParseContext<'_>, node: &XmlNode) -> Option<Dependency> {
    if node.name == "version" {
        Some(project_version_dependency(context.text, node))
    } else {
        package_dependency(context, node)
    }
}
