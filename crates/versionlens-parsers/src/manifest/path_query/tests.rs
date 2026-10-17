use super::{PathNode, query};

/// A small document tree that stands in for a parsed format.
enum Tree {
    Leaf,
    Map(Vec<(String, Tree)>),
    List(Vec<Tree>),
}

fn map(entries: Vec<(&str, Tree)>) -> Tree {
    Tree::Map(
        entries
            .into_iter()
            .map(|(key, child)| (key.to_owned(), child))
            .collect(),
    )
}

impl<'a> PathNode for &'a Tree {
    fn child(&self, key: &str) -> Option<Self> {
        match *self {
            Tree::Map(entries) => entries
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, child)| child),
            Tree::Leaf | Tree::List(_) => None,
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
