use std::ops::Range as ByteRange;

use versionlens_model::Range;

use crate::positions::offset_range;

mod collect;

pub(super) use collect::collect_nodes;

pub(super) fn direct_children<'a>(parent: &XmlNode, nodes: &'a [XmlNode]) -> Vec<&'a XmlNode> {
    nodes
        .iter()
        .filter(|node| {
            node.open_start > parent.open_start
                && node.close_end < parent.close_end
                && node
                    .path
                    .rsplit_once('.')
                    .is_some_and(|(path, _)| path == parent.path)
        })
        .collect()
}

pub(super) fn child_named<'a>(children: &[&'a XmlNode], name: &str) -> Option<&'a XmlNode> {
    children.iter().copied().find(|node| node.name == name)
}

pub(super) fn texts_from_nodes(nodes: &[XmlNode], path: &str) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| node.path == path && !node.text.is_empty())
        .map(|node| node.text.as_str().to_owned())
        .collect()
}

pub(super) fn text_range(text: &str, node: &XmlNode) -> Range {
    let range = node
        .text_range
        .as_ref()
        .map_or(node.open_start..node.open_start, |range| {
            range.start..range.end
        });
    let value_start = text[range.start..range.end]
        .find(node.text.as_str())
        .map_or(range.start, |offset| range.start + offset);
    offset_range(text, value_start, value_start + node.text.len())
}

pub(super) struct XmlNode {
    pub(super) name: String,
    pub(super) path: String,
    pub(super) open_start: usize,
    pub(super) close_end: usize,
    pub(super) text: String,
    pub(super) text_range: Option<ByteRange<usize>>,
}
