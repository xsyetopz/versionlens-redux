use super::{DocumentInput, Ecosystem, parse_document, parse_document_with_dependency_paths};
use crate::document::test_support::extract_range;

#[test]
fn parses_dub_json_dependency_groups() {
    let text = r#"{
  "dependencies": {
    "vibe-d": "~>0.9.7",
    "painlessjson": { "version": "1.4.0" },
    "local": { "path": "../local" },
    "remote": { "repository": "git@example.com:org/repo.git" }
  },
  "versions": {
    "imageformats": "1.0.0"
  },
  "subPackages": [
    "./modules/selector",
    {
      "standardpaths": "~>0.2.1"
    }
  ]
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Dub);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "vibe-d");
    assert_eq!(dependencies[0].requirement, "~>0.9.7");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "~>0.9.7"
    );
    assert_eq!(dependencies[1].name, "painlessjson");
    assert_eq!(dependencies[1].requirement, "1.4.0");
    assert_eq!(dependencies[2].group, "versions");
    assert_eq!(dependencies[2].name, "imageformats");
    assert_eq!(dependencies[2].requirement, "1.0.0");
}

#[test]
fn parses_configured_dub_subpackages() {
    let text = r#"{
  "dependencies": {
    "vibe-d": "~>0.9.7"
  },
  "subPackages": [
    "./modules/selector",
    {
      "standardpaths": "~>0.2.1"
    }
  ]
}"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/dub.json".to_owned(),
            language_id: "json".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["dependencies", "versions", "subPackages"],
    );

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "vibe-d");
    assert_eq!(dependencies[0].requirement, "~>0.9.7");
    assert_eq!(dependencies[1].group, "subPackages");
    assert_eq!(dependencies[1].name, "standardpaths");
    assert_eq!(dependencies[1].requirement, "~>0.2.1");
}

#[test]
fn parses_dub_selections_versions() {
    let text = r#"{
  "fileVersion": 1,
  "versions": {
    "gtk-d:gtkd": "3.11.0",
    "imageformats": "7.0.2"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.selections.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Dub);
    assert_eq!(dependencies[0].group, "versions");
    assert_eq!(dependencies[0].name, "gtk-d:gtkd");
    assert_eq!(dependencies[0].requirement, "3.11.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "3.11.0"
    );
}
