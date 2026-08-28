use crate::support;
use marked_yaml::types::MarkedMappingNode;

use versionlens_model::Dependency;

mod build;
mod dependencies;
mod image;

use dependencies::mapping_node_dependencies;

pub(crate) fn parse_docker_compose_yaml(text: &str) -> Vec<Dependency> {
    support::with_yaml_mapping(text, |root| {
        let service_dependencies = match root.get_node("services") {
            Some(marked_yaml::types::Node::Mapping(services)) => services
                .iter()
                .flat_map(|(_, service)| mapping_node_dependencies(text, service))
                .collect(),
            _ => vec![],
        };

        service_dependencies
            .into_iter()
            .chain(extension_dependencies(text, root))
            .collect()
    })
    .unwrap_or_default()
}

fn extension_dependencies(text: &str, root: &MarkedMappingNode) -> Vec<Dependency> {
    root.iter()
        .filter(|(key, _)| key.as_str().starts_with("x-"))
        .flat_map(|(_, value)| mapping_node_dependencies(text, value))
        .collect()
}
