use marked_yaml::types::{MarkedMappingNode, Node};

use crate::model::Dependency;

use super::build::build_dependency;
use super::image::image_dependency;

pub(super) fn service_dependencies(text: &str, service: &MarkedMappingNode) -> Vec<Dependency> {
    if let Some(dependency) = service
        .get_scalar("image")
        .and_then(|image| image_dependency(text, image))
    {
        return vec![dependency];
    }

    service
        .get_node("build")
        .and_then(|build| build_dependency(text, build))
        .into_iter()
        .collect()
}

pub(super) fn mapping_node_dependencies(text: &str, service: &Node) -> Vec<Dependency> {
    match service {
        Node::Mapping(service) => service_dependencies(text, service),
        Node::Scalar(_) | Node::Sequence(_) => Vec::new(),
    }
}
