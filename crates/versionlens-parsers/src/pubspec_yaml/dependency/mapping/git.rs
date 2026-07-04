use marked_yaml::types::{MarkedMappingNode, MarkedScalarNode, Node};

pub(super) fn git_value(map: &MarkedMappingNode) -> Option<&MarkedScalarNode> {
    match map.get_node("git")? {
        Node::Scalar(value) => Some(value),
        Node::Mapping(git) => git.get_scalar("url"),
        Node::Sequence(_) => None,
    }
}
