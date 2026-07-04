use super::{DocumentInput, Ecosystem, parse_document, parse_document_with_dependency_paths};
use crate::document::test_support::extract_range;

#[test]
fn parses_package_json_dependency_groups() {
    let text = r#"{
    "dependencies": {
    "@types/node": "26.0.1",
    "aliased": "npm:pacote@11.1.9",
    "ranged-alias": "npm:@types/react@^19.0.0",
    "unversioned-alias": "npm:types-react",
    "unversioned-scoped-alias": "npm:@types/react",
    "empty-alias": "npm:pacote@",
    "local-file": "file:../local",
    "local": "link:../local",
    "workspace-only": "workspace:*",
    "catalog-only": "catalog:"
  },
  "devDependencies": {
    "typescript": "6.0.3"
  },
  "peerDependencies": {
    "vscode": "^1.75.0"
  },
  "bundledDependencies": {
    "bundled": "1.0.0"
  },
  "bundleDependencies": {
    "bundled-alias": "1.1.0"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 9);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "@types/node");
    assert_eq!(dependencies[0].requirement, "26.0.1");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "26.0.1"
    );
    assert_eq!(dependencies[1].name, "pacote");
    assert_eq!(dependencies[1].requirement, "11.1.9");
    assert_eq!(dependencies[1].requirement_prefix, "npm:pacote@");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "npm:pacote@11.1.9"
    );
    assert_eq!(dependencies[2].name, "@types/react");
    assert_eq!(dependencies[2].requirement, "^19.0.0");
    assert_eq!(dependencies[2].requirement_prefix, "npm:@types/react@");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "npm:@types/react@^19.0.0"
    );
    assert_eq!(dependencies[3].name, "types-react");
    assert_eq!(dependencies[3].requirement, "");
    assert_eq!(dependencies[3].requirement_prefix, "npm:types-react@");
    assert_eq!(extract_range(text, dependencies[3].requirement_range), "");
    assert_eq!(dependencies[4].name, "@types/react");
    assert_eq!(dependencies[4].requirement, "");
    assert_eq!(dependencies[4].requirement_prefix, "npm:@types/react@");
    assert_eq!(extract_range(text, dependencies[4].requirement_range), "");
    assert_eq!(dependencies[5].name, "local-file");
    assert_eq!(dependencies[5].requirement, "file:../local");
    assert_eq!(
        extract_range(text, dependencies[5].requirement_range),
        "file:../local"
    );
    assert_eq!(dependencies[6].name, "local");
    assert_eq!(dependencies[6].requirement, "file:../local/package.json");
    assert_eq!(
        extract_range(text, dependencies[6].requirement_range),
        "link:../local"
    );
    assert_eq!(dependencies[7].group, "devDependencies");
    assert_eq!(dependencies[8].group, "peerDependencies");
}

#[test]
fn package_json_ranges_count_utf16_code_units_before_dependencies() {
    let text = r#"{"emoji":"😀","dependencies":{"left-pad":"1.0.0"}}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "left-pad");
    assert_eq!(extract_range(text, dependencies[0].range), "left-pad");
    assert_eq!(dependencies[0].range.start.character, 31);
}

#[test]
fn parses_configured_npm_bundle_dependency_name_arrays() {
    let text = r#"{
  "bundledDependencies": ["left-pad"],
  "bundleDependencies": ["right-pad"]
}"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["bundledDependencies", "bundleDependencies"],
    );

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "bundledDependencies");
    assert_eq!(dependencies[0].name, "left-pad");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(extract_range(text, dependencies[0].range), "left-pad");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[1].group, "bundleDependencies");
    assert_eq!(dependencies[1].name, "right-pad");
    assert_eq!(dependencies[1].requirement, "");
}

#[test]
fn parses_package_manager_prerelease_build_metadata() {
    let text = r#"{
  "packageManager": "pnpm@9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac"
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "packageManager");
    assert_eq!(dependencies[0].name, "pnpm");
    assert_eq!(
        dependencies[0].requirement,
        "9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac"
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac"
    );
}

#[test]
fn ignores_invalid_package_manager_values() {
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{
  "packageManager": "@9.1.2",
  "dependencies": {
    "left-pad": "1.0.0"
  }
}"#
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "left-pad");
}
