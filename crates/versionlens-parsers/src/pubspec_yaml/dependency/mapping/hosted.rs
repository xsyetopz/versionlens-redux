use marked_yaml::types::{MarkedMappingNode, Node};

pub(super) fn hosted_url(map: &MarkedMappingNode) -> Option<String> {
    match map.get_node("hosted")? {
        Node::Scalar(value) => Some(value.as_str().to_owned()),
        Node::Mapping(hosted) => hosted
            .get_scalar("url")
            .map(|value| value.as_str().to_owned()),
        Node::Sequence(_) => None,
    }
    .filter(|url| !url.is_empty())
}

pub(super) fn hosted_name(map: &MarkedMappingNode) -> Option<String> {
    match map.get_node("hosted")? {
        Node::Mapping(hosted) => hosted
            .get_scalar("name")
            .map(|value| value.as_str().to_owned()),
        Node::Scalar(_) | Node::Sequence(_) => None,
    }
    .filter(|name| !name.is_empty())
}
