use marked_yaml::types::{MarkedMappingNode, Node};

pub(super) fn mapping_node(node: &Node) -> Option<&MarkedMappingNode> {
    let Node::Mapping(mapping) = node else {
        return None;
    };
    Some(mapping)
}
