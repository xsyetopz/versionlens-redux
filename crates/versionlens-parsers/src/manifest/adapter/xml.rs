use std::ops::Range as ByteRange;

use versionlens_model::Range;

use crate::manifest::path_query::PathNode;
use crate::positions::offset_range;

mod collect;

pub(crate) use collect::collect_nodes;

pub(crate) fn direct_children<'a>(parent: &XmlNode, nodes: &'a [XmlNode]) -> Vec<&'a XmlNode> {
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

pub(crate) fn child_named<'a>(children: &[&'a XmlNode], name: &str) -> Option<&'a XmlNode> {
    children.iter().copied().find(|node| node.name == name)
}

pub(crate) fn texts_from_nodes(nodes: &[XmlNode], path: &str) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| node.path == path && !node.text.is_empty())
        .map(|node| node.text.as_str().to_owned())
        .collect()
}

pub(crate) fn text_range(text: &str, node: &XmlNode) -> Range {
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

/// An element with the byte span of its tags and of its text. `path` joins
/// the names of its ancestors and its own with `.`.
pub(crate) struct XmlNode {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) open_start: usize,
    pub(crate) close_end: usize,
    pub(crate) text: String,
    pub(crate) text_range: Option<ByteRange<usize>>,
}

/// An element of a parsed document as a node of a path query. The root stands
/// above the document element, so a path starts with the document element's
/// name, such as `project.dependencies.dependency`.
#[derive(Clone, Copy)]
pub(crate) struct XmlElement<'a> {
    nodes: &'a [XmlNode],
    index: Option<usize>,
}

impl<'a> XmlElement<'a> {
    pub(crate) fn root(nodes: &'a [XmlNode]) -> Self {
        Self { nodes, index: None }
    }

    /// The position of the element in the collected nodes, which are in the
    /// order that their closing tags appear. The root has none.
    pub(crate) fn index(&self) -> Option<usize> {
        self.index
    }

    fn children_matching(&self, key: Option<&str>) -> Vec<(String, Self)> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| match self.index {
                None => !node.path.contains('.'),
                Some(parent) => {
                    let parent = &self.nodes[parent];
                    node.open_start > parent.open_start
                        && node.close_end < parent.close_end
                        && node
                            .path
                            .rsplit_once('.')
                            .is_some_and(|(path, _)| path == parent.path)
                }
            })
            .filter(|(_, node)| key.is_none_or(|key| node.name == key))
            .map(|(index, node)| {
                (
                    node.name.clone(),
                    Self {
                        nodes: self.nodes,
                        index: Some(index),
                    },
                )
            })
            .collect()
    }
}

impl PathNode for XmlElement<'_> {
    fn children_named(&self, key: &str) -> Vec<Self> {
        self.children_matching(Some(key))
            .into_iter()
            .map(|(_, child)| child)
            .collect()
    }

    fn children(&self) -> Vec<(String, Self)> {
        self.children_matching(None)
    }
}
