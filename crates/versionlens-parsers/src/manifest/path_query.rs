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
mod tests {
    use super::{PathNode, query, selects};

    /// A small document tree that stands in for a parsed format.
    enum Tree {
        Leaf,
        Map(Vec<(String, Self)>),
        List(Vec<Self>),
    }

    fn map(entries: Vec<(&str, Tree)>) -> Tree {
        Tree::Map(
            entries
                .into_iter()
                .map(|(key, child)| (key.to_owned(), child))
                .collect(),
        )
    }

    impl PathNode for &Tree {
        fn children_named(&self, key: &str) -> Vec<Self> {
            match *self {
                Tree::Map(entries) => entries
                    .iter()
                    .filter(|(name, _)| name == key)
                    .map(|(_, child)| child)
                    .collect(),
                Tree::Leaf | Tree::List(_) => Vec::new(),
            }
        }

        fn children(&self) -> Vec<(String, Self)> {
            match *self {
                Tree::Map(entries) => entries
                    .iter()
                    .map(|(name, child)| (name.clone(), child))
                    .collect(),
                Tree::List(items) => items
                    .iter()
                    .enumerate()
                    .map(|(index, child)| (index.to_string(), child))
                    .collect(),
                Tree::Leaf => Vec::new(),
            }
        }
    }

    fn sample() -> Tree {
        map(vec![
            (
                "deps",
                map(vec![("serde", Tree::Leaf), ("tokio", Tree::Leaf)]),
            ),
            (
                "target",
                map(vec![
                    (
                        "linux",
                        map(vec![("dependencies", map(vec![("libc", Tree::Leaf)]))]),
                    ),
                    (
                        "cfg",
                        map(vec![("build-dependencies", map(vec![("cc", Tree::Leaf)]))]),
                    ),
                ]),
            ),
            (
                "list",
                Tree::List(vec![map(vec![("name", Tree::Leaf)]), Tree::Leaf]),
            ),
        ])
    }

    fn matched_keys(root: &Tree, segments: &[&str]) -> Vec<Vec<String>> {
        query(root, segments)
            .into_iter()
            .map(|found| found.keys)
            .collect()
    }

    #[test]
    fn literal_path_matches_the_node_it_names() {
        let root = sample();
        let found = query(&root, &["deps", "serde"]);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].keys, ["deps", "serde"]);
        assert!(matches!(found[0].node, Tree::Leaf));
    }

    #[test]
    fn empty_path_matches_the_root() {
        let root = sample();

        assert_eq!(matched_keys(&root, &[]), vec![Vec::<String>::new()]);
    }

    #[test]
    fn star_matches_every_child_and_keys_each_one() {
        let root = sample();

        assert_eq!(
            matched_keys(&root, &["deps", "*"]),
            vec![
                vec!["deps".to_owned(), "serde".to_owned()],
                vec!["deps".to_owned(), "tokio".to_owned()],
            ]
        );
    }

    #[test]
    fn star_in_the_middle_expands_before_the_rest_of_the_path() {
        let root = sample();

        assert_eq!(
            matched_keys(&root, &["target", "*", "dependencies"]),
            vec![vec![
                "target".to_owned(),
                "linux".to_owned(),
                "dependencies".to_owned(),
            ]]
        );
    }

    #[test]
    fn several_stars_each_expand() {
        let root = sample();

        assert_eq!(
            matched_keys(&root, &["target", "*", "*"]),
            vec![
                vec![
                    "target".to_owned(),
                    "linux".to_owned(),
                    "dependencies".to_owned(),
                ],
                vec![
                    "target".to_owned(),
                    "cfg".to_owned(),
                    "build-dependencies".to_owned(),
                ],
            ]
        );
    }

    #[test]
    fn list_children_are_keyed_by_index() {
        let root = sample();

        assert_eq!(
            matched_keys(&root, &["list", "*"]),
            vec![
                vec!["list".to_owned(), "0".to_owned()],
                vec!["list".to_owned(), "1".to_owned()],
            ]
        );
    }

    #[test]
    fn missing_key_matches_nothing() {
        let root = sample();

        assert!(query(&root, &["deps", "missing"]).is_empty());
    }

    #[test]
    fn star_over_a_leaf_matches_nothing() {
        let root = sample();

        assert!(query(&root, &["deps", "serde", "*"]).is_empty());
    }

    #[test]
    fn selects_a_key_path_that_the_query_matches_key_for_key() {
        assert!(selects(&["deps", "serde"], &["deps", "serde"]));
        assert!(selects(&["deps", "*"], &["deps", "tokio"]));
        assert!(selects(
            &["target", "*", "dependencies"],
            &["target", "linux", "dependencies"]
        ));
    }

    #[test]
    fn selects_rejects_a_key_path_of_another_length_or_key() {
        assert!(!selects(&["deps"], &["deps", "serde"]));
        assert!(!selects(&["deps", "serde"], &["deps"]));
        assert!(!selects(&["deps", "*"], &["deps"]));
        assert!(!selects(&["deps", "*"], &["other", "serde"]));
    }

    #[test]
    fn selects_everything_below_a_query_that_ends_in_star() {
        assert!(selects(&["deps", "*"], &["deps", "serde", "features"]));
        assert!(selects(
            &["workspace", "metadata", "*"],
            &["workspace", "metadata", "tool", "plugins"]
        ));
        assert!(!selects(&["deps", "*"], &["other", "serde", "features"]));
    }

    #[test]
    fn selects_nothing_below_a_query_that_does_not_end_in_star() {
        assert!(!selects(&["deps"], &["deps", "serde", "features"]));
        assert!(!selects(
            &["target", "*", "dependencies"],
            &["target", "linux", "dependencies", "libc"]
        ));
    }
}
