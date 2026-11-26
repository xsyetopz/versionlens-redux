//! The original `dependencyProperties` setting: upstream 1.3.0 read a manifest only at the paths
//! it lists, and read the keys of a listed object no rule covers.

use crate::catalog::file_type_of;
use crate::extract::{extract, properties};

fn read(path: &str, text: &str, paths: &[&str]) -> Vec<(String, String)> {
    let file = file_type_of(path, text).unwrap();
    let paths: Vec<String> = paths.iter().map(|p| (*p).to_owned()).collect();
    properties(file, text, extract(file, text), &paths)
        .into_iter()
        .map(|d| (d.name, d.req))
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(n, r)| ((*n).to_owned(), (*r).to_owned()))
        .collect()
}

#[test]
fn keeps_only_dependencies_under_listed_paths() {
    // Arrange
    let text = "{\n  \"dependencies\": { \"a\": \"1.0.0\" },\n  \"devDependencies\": { \"b\": \"2.0.0\" }\n}\n";

    // Act
    let deps = read("/work/package.json", text, &["dependencies"]);

    // Assert
    assert_eq!(deps, pairs(&[("a", "1.0.0")]));
}

#[test]
fn reads_keys_of_listed_object_no_rule_covers() {
    // Arrange: `resolutions` is Yarn's, which the npm row reads through its own rules, so a
    // made-up `pinned` object stands for a path only the setting names.
    let text =
        "{\n  \"dependencies\": { \"a\": \"1.0.0\" },\n  \"pinned\": { \"c\": \"3.0.0\" }\n}\n";

    // Act
    let deps = read("/work/package.json", text, &["dependencies", "pinned"]);

    // Assert
    assert_eq!(deps, pairs(&[("a", "1.0.0"), ("c", "3.0.0")]));
}

#[test]
fn matches_wildcard_segments_in_toml() {
    // Arrange: upstream's default cargo list holds `dependencies.*`.
    let text = "[dependencies]\nserde = \"1.0\"\n\n[dev-dependencies]\ninsta = \"1.0\"\n";

    // Act
    let deps = read(
        "/work/Cargo.toml",
        text,
        &["dependencies", "dependencies.*"],
    );

    // Assert
    assert_eq!(deps, pairs(&[("serde", "1.0")]));
}

#[test]
fn keeps_every_dependency_of_line_read_manifest() {
    // Arrange
    let text = "requests==2.31.0\n";

    // Act
    let deps = read("/work/requirements.txt", text, &["project"]);

    // Assert
    assert_eq!(deps, pairs(&[("requests", "==2.31.0")]));
}
