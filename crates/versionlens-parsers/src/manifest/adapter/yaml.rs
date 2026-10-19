//! The YAML adapter: a [`PathNode`] tree over a marked document. Scalar and
//! key spans come from the marks, and exclude any quotes.

use std::ops::Range;

use marked_yaml::types::{MarkedScalarNode, Node};

use super::Scalar;
use crate::manifest::path_query::PathNode;
use crate::yaml::scalar_range;

/// A YAML node of a path query. Mappings are walked by key and sequences by
/// index.
#[derive(Clone, Copy)]
pub(crate) struct YamlNode<'a> {
    text: &'a str,
    node: &'a Node,
    key: Option<&'a MarkedScalarNode>,
}

impl<'a> YamlNode<'a> {
    pub(crate) fn root(text: &'a str, document: &'a Node) -> Self {
        Self {
            text,
            node: document,
            key: None,
        }
    }

    /// The scalar the node holds, with the span of its content.
    pub(crate) fn scalar(&self) -> Option<Scalar> {
        let scalar = self.node.as_scalar()?;
        Some(Scalar {
            value: scalar.as_str().to_owned(),
            span: scalar_range(self.text, scalar)?,
        })
    }

    /// The span of the key that holds the node, absent for the root and for
    /// sequence elements.
    pub(crate) fn key_span(&self) -> Option<Range<usize>> {
        scalar_range(self.text, self.key?)
    }
}

impl PathNode for YamlNode<'_> {
    fn children_named(&self, key: &str) -> Vec<Self> {
        match self.node {
            Node::Mapping(mapping) => mapping
                .iter()
                .filter(|(name, _)| name.as_str() == key)
                .map(|(name, node)| Self {
                    text: self.text,
                    node,
                    key: Some(name),
                })
                .collect(),
            Node::Sequence(sequence) => key
                .parse::<usize>()
                .ok()
                .and_then(|index| sequence.get(index))
                .map(|node| Self {
                    text: self.text,
                    node,
                    key: None,
                })
                .into_iter()
                .collect(),
            Node::Scalar(_) => Vec::new(),
        }
    }

    fn children(&self) -> Vec<(String, Self)> {
        match self.node {
            Node::Mapping(mapping) => mapping
                .iter()
                .map(|(name, node)| {
                    (
                        name.as_str().to_owned(),
                        Self {
                            text: self.text,
                            node,
                            key: Some(name),
                        },
                    )
                })
                .collect(),
            Node::Sequence(sequence) => sequence
                .iter()
                .enumerate()
                .map(|(index, node)| {
                    (
                        index.to_string(),
                        Self {
                            text: self.text,
                            node,
                            key: None,
                        },
                    )
                })
                .collect(),
            Node::Scalar(_) => Vec::new(),
        }
    }
}
