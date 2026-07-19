use marked_yaml::types::Node::Mapping as YamlMapping;
use marked_yaml::{parse_yaml, types::MarkedMappingNode};

use versionlens_model::Dependency;

mod build;
mod dependencies;
mod image;

use dependencies::mapping_node_dependencies;

pub(crate) fn parse_docker_compose_yaml(text: &str) -> Vec<Dependency> {
    let Ok(root) = parse_yaml(0, text) else {
        return vec![];
    };
    let Some(root) = root.as_mapping() else {
        return vec![];
    };
    let service_dependencies = match root.get_node("services") {
        Some(YamlMapping(services)) => services
            .iter()
            .flat_map(|(_, service)| mapping_node_dependencies(text, service))
            .collect(),
        _ => vec![],
    };

    let extension_dependencies = extension_dependencies(text, root);

    service_dependencies
        .into_iter()
        .chain(extension_dependencies)
        .collect()
}

fn extension_dependencies(text: &str, root: &MarkedMappingNode) -> Vec<Dependency> {
    root.iter()
        .filter(|(key, _)| key.as_str().starts_with("x-"))
        .flat_map(|(_, value)| mapping_node_dependencies(text, value))
        .collect()
}
