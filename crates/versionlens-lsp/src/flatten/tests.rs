use super::*;

fn find<'a>(leaves: &'a [Leaf], path: &str) -> &'a Leaf {
    leaves
        .iter()
        .find(|l| l.path.join(".") == path)
        .unwrap_or_else(|| panic!("no {path} in {leaves:#?}"))
}

fn check(format: Format, text: &str, path: &str, value: &str) {
    let leaves = flatten(format, text);
    let leaf = find(&leaves, path);
    assert_eq!(leaf.value, value);
    assert_eq!(&text[leaf.span.clone()], value, "span of {path}");
}

#[test]
fn reads_json_with_comments() {
    let text = "{\n  // c\n  \"dependencies\": { \"a\": \"^1.0.0\", },\n  \"list\": [\"x\"]\n}";
    check(Format::Json, text, "dependencies.a", "^1.0.0");
    check(Format::Json, text, "list.0", "x");
}

#[test]
fn reads_toml_tables_and_inline_tables() {
    let text = "[dependencies]\nserde = { version = \"1.0\", features = [\"derive\"] }\nlog = '0.4'\n[[bin]]\nname = \"x\"\n";
    check(Format::Toml, text, "dependencies.serde.version", "1.0");
    check(Format::Toml, text, "dependencies.log", "0.4");
    check(Format::Toml, text, "bin.0.name", "x");
}

#[test]
fn reads_yaml_maps_lists_and_quotes() {
    let text = "dependencies:\n  - name: redis # c\n    version: \"17.0.0\"\n  - name: pg\n    version: 12.1.0\nscript: |\n  key: no\nimages:\n- name: nginx\n";
    let leaves = flatten(Format::Yaml, text);
    assert_eq!(find(&leaves, "dependencies.0.name").value, "redis");
    assert_eq!(find(&leaves, "dependencies.1.version").value, "12.1.0");
    assert_eq!(
        find(&leaves, "dependencies.0.name").parent,
        find(&leaves, "dependencies.0.version").parent
    );
    assert!(
        leaves
            .iter()
            .all(|l| l.path.first().is_none_or(|p| p != "script"))
    );
    check(Format::Yaml, text, "dependencies.0.version", "17.0.0");
    check(Format::Yaml, text, "images.0.name", "nginx");
}

#[test]
fn reads_yaml_mappings_behind_anchor() {
    let text =
        "x-common: &common\n  image: redis:7.2\nservices:\n  a:\n    <<: *common\n    tag: &t v1\n";
    check(Format::Yaml, text, "x-common.image", "redis:7.2");
    let leaves = flatten(Format::Yaml, text);
    assert!(leaves.iter().all(|l| l.path.join(".") != "services.a.tag"));
}

#[test]
fn reads_yaml_flow_collections() {
    // Arrange: yaml.org/spec/1.2.2/#74-flow-collection-styles, across lines and with comments.
    let text = "dependencies: {http: ^1.0.0, 'path': \"1.9.0\", bare: }\nchannels: [conda-forge]\nlist:\n  - [numpy=1.26, {pip: [requests==2.31.0]}] # c\nnext: {a: [\n    x, # c\n    y]}\nafter: 2.0.0\n";

    // Act
    let leaves = flatten(Format::Yaml, text);

    // Assert
    check(Format::Yaml, text, "dependencies.http", "^1.0.0");
    check(Format::Yaml, text, "dependencies.path", "1.9.0");
    assert_eq!(find(&leaves, "dependencies.bare").value, "");
    check(Format::Yaml, text, "channels.0", "conda-forge");
    check(Format::Yaml, text, "list.0.0", "numpy=1.26");
    check(Format::Yaml, text, "list.0.1.pip.0", "requests==2.31.0");
    check(Format::Yaml, text, "next.a.1", "y");
    check(Format::Yaml, text, "after", "2.0.0");
    assert_eq!(
        find(&leaves, "dependencies.http").parent,
        find(&leaves, "dependencies.path").parent
    );
}

#[test]
fn reads_block_yaml_below_unclosed_flow_collection() {
    // Arrange: an editor's buffer while `{` is being typed.
    let text = "dependencies: {http: ^1.0.0\ndev_dependencies:\n  lints: ^5.0.0\n";

    // Act
    let leaves = flatten(Format::Yaml, text);

    // Assert
    assert!(leaves.iter().all(|l| l.path[0] != "dependencies"));
    check(Format::Yaml, text, "dev_dependencies.lints", "^5.0.0");
}

#[test]
fn reads_xml_elements_and_attributes() {
    let text = "<?xml version=\"1.0\"?>\n<Project><!-- <x/> --><ItemGroup><PackageReference Include=\"A\" Version=\"1.0\" />\n<PackageReference Include=\"B\"><Version> 2.0 </Version></PackageReference></ItemGroup></Project>";
    let leaves = flatten(Format::Xml, text);
    let attrs: Vec<&str> = leaves.iter().map(|l| l.value.as_str()).collect();
    assert_eq!(attrs, ["A", "1.0", "B", "2.0"]);
    assert_eq!(leaves[2].parent, leaves[3].parent);
    check(
        Format::Xml,
        text,
        "Project.ItemGroup.PackageReference.Version",
        "2.0",
    );
}

#[test]
fn reads_ini_sections_and_plugin_names() {
    let text = "name = Dist-Zilla\n; comment\n[Prereqs / TestRequires]\nTest::More = 0.90 ; why\n[Prereqs]\nperl = v5.22.0\n";
    let leaves = flatten(Format::Ini, text);
    let paths: Vec<String> = leaves.iter().map(|l| l.path.join(".")).collect();
    assert_eq!(
        paths,
        ["name", "Prereqs.TestRequires.Test::More", "Prereqs.perl"]
    );
    check(Format::Ini, text, "Prereqs.TestRequires.Test::More", "0.90");
    assert_ne!(leaves[1].parent, leaves[2].parent);
}
