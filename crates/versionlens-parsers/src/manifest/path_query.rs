//! A path-query engine over any keyed document tree. A query is a list of
//! segments: a `*` segment matches every child of the node it stands on, and
//! any other segment matches the children stored under that key. Format
//! adapters implement [`PathNode`]; the callers name the groups of the
//! dependencies they collect from the keys of each match.

/// A node of a parsed document that a path query can walk.
pub(crate) trait PathNode: Sized {
    /// The children stored under `key`: none, one, or every element of that
    /// name where the format allows a name to repeat.
    fn children_named(&self, key: &str) -> Vec<Self>;

    /// Every child, paired with the key that a `*` segment reports for it.
    fn children(&self) -> Vec<(String, Self)>;
}

/// A node that a query matched, with one key per segment that led to it.
pub(crate) struct PathMatch<N> {
    pub(crate) keys: Vec<String>,
    pub(crate) node: N,
}

/// Every node that `segments` reaches from `root`, in document order.
pub(crate) fn query<N: PathNode>(root: N, segments: &[&str]) -> Vec<PathMatch<N>> {
    let mut matches = Vec::new();
    walk(root, segments, &mut Vec::new(), &mut matches);
    matches
}

/// Whether `segments` selects the key path `keys`. The query matches the path
/// key for key. A query that ends in `*` also selects every path below its
/// matches, so a `*` leaf selects the keys of the tables under each match.
pub(crate) fn selects(segments: &[&str], keys: &[&str]) -> bool {
    let within_depth = if segments.last() == Some(&"*") {
        keys.len() >= segments.len()
    } else {
        keys.len() == segments.len()
    };

    within_depth
        && segments
            .iter()
            .zip(keys)
            .all(|(segment, key)| *segment == "*" || segment == key)
}

fn walk<N: PathNode>(
    node: N,
    segments: &[&str],
    keys: &mut Vec<String>,
    matches: &mut Vec<PathMatch<N>>,
) {
    let Some((segment, rest)) = segments.split_first() else {
        matches.push(PathMatch {
            keys: keys.clone(),
            node,
        });
        return;
    };

    if *segment == "*" {
        for (key, child) in node.children() {
            keys.push(key);
            walk(child, rest, keys, matches);
            keys.pop();
        }
    } else {
        for child in node.children_named(segment) {
            keys.push((*segment).to_owned());
            walk(child, rest, keys, matches);
            keys.pop();
        }
    }
}

#[cfg(test)]
mod tests;
