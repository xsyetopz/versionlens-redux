use std::collections::HashMap;

use super::XmlNode;

pub(super) fn collect_properties(nodes: &[XmlNode]) -> HashMap<String, usize> {
    let mut properties = HashMap::new();
    for (index, node) in nodes.iter().enumerate() {
        if node.path.starts_with("project.properties.")
            && !properties.contains_key(node.name.as_str())
        {
            properties.insert(node.name.as_str().to_owned(), index);
        }
    }
    properties
}

pub(super) fn resolve_property<'a>(
    node: &'a XmlNode,
    nodes: &'a [XmlNode],
    properties: &HashMap<String, usize>,
) -> &'a XmlNode {
    node.text
        .strip_prefix("${")
        .and_then(|value| value.strip_suffix('}'))
        .and_then(|name| properties.get(name))
        .and_then(|index| nodes.get(*index))
        .unwrap_or(node)
}
