use marked_yaml::types::MarkedScalarNode;
use marked_yaml::types::Node;
use marked_yaml::types::Node::{
    Mapping as YamlMapping, Scalar as YamlScalar, Sequence as YamlSequence,
};

use versionlens_model::Dependency;

mod mapping;
mod scalar;
mod source;

use mapping::mapping_dependency;
use scalar::scalar_dependency_from_source;
use source::PubspecDependencySource;

pub(super) fn dependency_from_node(
    text: &str,
    group: &str,
    key: &MarkedScalarNode,
    value: &Node,
) -> Option<Dependency> {
    let source = PubspecDependencySource { text, group, key };
    match value {
        YamlScalar(value) => scalar_dependency_from_source(&source, value),
        YamlMapping(map) => mapping_dependency(&source, map),
        YamlSequence(_) => None,
    }
}

pub(super) fn scalar_dependency(
    text: &str,
    group: &str,
    key: &MarkedScalarNode,
    value: &MarkedScalarNode,
) -> Option<Dependency> {
    let source = PubspecDependencySource { text, group, key };
    scalar_dependency_from_source(&source, value)
}
