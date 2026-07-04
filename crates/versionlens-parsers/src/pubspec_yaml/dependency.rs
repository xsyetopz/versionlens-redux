use marked_yaml::types::{MarkedScalarNode, Node};

use crate::model::Dependency;

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
        Node::Scalar(value) => scalar_dependency_from_source(&source, value),
        Node::Mapping(map) => mapping_dependency(&source, map),
        Node::Sequence(_) => None,
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
