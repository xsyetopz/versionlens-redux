use super::*;

#[test]
fn typst_compiler_asks_git_and_crates_io() {
    let registry = registry::get("typst-compiler").unwrap();

    let asked = queries(registry, "compiler", None, "0.13.0");

    let pairs: Vec<(&str, &str, &str)> = asked
        .iter()
        .map(|q| (q.ecosystem, q.name.as_str(), q.version.as_str()))
        .collect();
    assert_eq!(
        pairs,
        [
            ("GIT", "https://github.com/typst/typst", "0.13.0"),
            ("crates.io", "typst-cli", "0.13.0"),
        ]
    );
}

#[test]
fn packer_plugin_asks_go_and_its_repository() {
    let registry = registry::get("packer").unwrap();

    let asked = queries(registry, "hashicorp/amazon", None, "v1.0.0");

    let names: Vec<(&str, &str)> = asked
        .iter()
        .map(|q| (q.ecosystem, q.name.as_str()))
        .collect();
    assert_eq!(
        names,
        [
            ("Go", "github.com/hashicorp/packer-plugin-amazon"),
            ("GIT", "https://github.com/hashicorp/packer-plugin-amazon"),
        ]
    );
}

#[test]
fn registry_without_also_asks_once() {
    let registry = registry::get("cargo").unwrap();

    let asked = queries(registry, "time", None, "0.1.43");

    assert_eq!(asked.len(), 1);
    assert_eq!(
        (asked[0].ecosystem, asked[0].name.as_str()),
        ("crates.io", "time")
    );
}
