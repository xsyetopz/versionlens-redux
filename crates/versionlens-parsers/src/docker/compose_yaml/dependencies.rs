use marked_yaml::types::MarkedMappingNode;
use marked_yaml::types::Node;
use marked_yaml::types::Node::{
    Mapping as YamlMapping, Scalar as YamlScalar, Sequence as YamlSequence,
};

use versionlens_model::Dependency;

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
        YamlMapping(service) => service_dependencies(text, service),
        YamlScalar(_) | YamlSequence(_) => vec![],
    }
}
