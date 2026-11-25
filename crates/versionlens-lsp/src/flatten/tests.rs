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
fn reads_yaml_mappings_behind_an_anchor() {
    let text =
        "x-common: &common\n  image: redis:7.2\nservices:\n  a:\n    <<: *common\n    tag: &t v1\n";
    check(Format::Yaml, text, "x-common.image", "redis:7.2");
    let leaves = flatten(Format::Yaml, text);
    assert!(leaves.iter().all(|l| l.path.join(".") != "services.a.tag"));
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
