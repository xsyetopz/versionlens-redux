use marked_yaml::{parse_yaml, types::Node};

use crate::model::Dependency;

mod build;
mod image;
mod service;

use service::mapping_node_dependencies;

pub(crate) fn parse_docker_compose_yaml(text: &str) -> Vec<Dependency> {
    let Ok(root) = parse_yaml(0, text) else {
        return Vec::new();
    };
    let Some(root) = root.as_mapping() else {
        return Vec::new();
    };
    let Some(Node::Mapping(services)) = root.get_node("services") else {
        return Vec::new();
    };

    services
        .iter()
        .flat_map(|(_, service)| mapping_node_dependencies(text, service))
        .collect()
}
