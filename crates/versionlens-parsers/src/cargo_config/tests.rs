use super::parse_cargo_config_registry_sources;

#[test]
fn parses_cargo_config_registry_sources() {
    let sources = parse_cargo_config_registry_sources(
        r#"
[registries.private]
index = "sparse+https://cargo.example.test/api/"

[registries]
mirror = { index = "https://mirror.example.test/api" }

[source.vendor]
registry = "https://vendor.example.test/index/"

[source.crates-io]
replace-with = "vendor"
"#,
    );

    assert_eq!(sources.len(), 4);
    assert_eq!(sources[0].name, "private");
    assert_eq!(sources[0].url, "https://cargo.example.test/api");
    assert_eq!(sources[0].replace_with, None);
    assert_eq!(sources[1].name, "mirror");
    assert_eq!(sources[1].url, "https://mirror.example.test/api");
    assert_eq!(sources[1].replace_with, None);
    let crates_io = sources
        .iter()
        .find(|source| source.name == "crates-io")
        .unwrap();
    let vendor = sources
        .iter()
        .find(|source| source.name == "vendor")
        .unwrap();
    assert_eq!(crates_io.url, "");
    assert_eq!(crates_io.replace_with.as_deref(), Some("vendor"));
    assert_eq!(vendor.url, "https://vendor.example.test/index");
    assert_eq!(vendor.replace_with, None);
}
