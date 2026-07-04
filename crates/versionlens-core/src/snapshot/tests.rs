use versionlens_parsers::{DocumentInput, parse_document};

use super::dependency_signature;

#[test]
fn dependency_signature_ignores_workspace_and_catalog_specs() {
    let left = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0","workspace-only":"workspace:*","catalog-only":"catalog:"}}"#
            .to_owned(),
        workspace_root: None,
    });
    let right = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0","workspace-only":"workspace:^","catalog-only":"catalog:next"}}"#
            .to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependency_signature(&left), dependency_signature(&right));
}

#[test]
fn dependency_signature_ignores_npm_package_manager_changes() {
    let left = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"packageManager":"pnpm@9.1.2","dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: None,
    });
    let right = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"packageManager":"pnpm@10.34.4","dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependency_signature(&left), dependency_signature(&right));
}
