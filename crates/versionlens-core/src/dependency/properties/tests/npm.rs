use super::{
    DocumentInput, Ecosystem, ManifestKind, session_with_properties, session_with_property_configs,
    session_with_scoped_property_configs,
};

#[test]
fn dependency_properties_are_filtered_in_rust() {
    let session = session_with_properties(Ecosystem::Npm, &["devDependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"},"devDependencies":{"is-odd":"1.0.0"}}"#
            .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "is-odd");
}

#[test]
fn dependency_properties_allow_custom_json_paths() {
    let session = session_with_properties(Ecosystem::Npm, &["resolutions"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"},"resolutions":{"is-even":"1.0.0"}}"#
            .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "is-even");
}

#[test]
fn dependency_properties_match_wildcard_groups() {
    let session =
        session_with_properties(Ecosystem::Npm, &["packageExtensions.*.peerDependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "\
packageExtensions:
  react@18:
    dependencies:
      scheduler: ^0.23.0
    peerDependencies:
      '@types/react': ^18.0.0
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "@types/react");
}

#[test]
fn deno_dependency_properties_filter_before_extraction() {
    let session = session_with_properties(Ecosystem::Deno, &["imports"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: r#"{
  "imports": { "@std/assert": "jsr:@std/assert@^1.0.0" },
  "scopes": { "https://deno.land/x/app/": { "@scope/pkg": "jsr:@scope/pkg@0.2.0" } }
}"#
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "@std/assert");
}

#[test]
fn dependency_properties_merge_same_ecosystem_configs() {
    let session = session_with_property_configs(&[
        (Ecosystem::Npm, &["devDependencies"][..]),
        (
            Ecosystem::Npm,
            &["packageExtensions.*.peerDependencies"][..],
        ),
    ]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "\
packageExtensions:
  react@18:
    peerDependencies:
      scheduler: ^0.23.0
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "scheduler");
}

#[test]
fn dependency_properties_allow_custom_pnpm_paths() {
    let session = session_with_properties(Ecosystem::Npm, &["customCatalog"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "\
catalog:
  react: ^18.0.0
customCatalog:
  scheduler: ^0.23.0
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "scheduler");
}

#[test]
fn npm_dependency_properties_do_not_disable_pnpm_yaml_defaults() {
    let session = session_with_scoped_property_configs(&[(
        Ecosystem::Npm,
        Some(ManifestKind::NpmPackageJson),
        &["devDependencies"],
    )]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "\
catalog:
  react: ^18.0.0
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "react");
}

#[test]
fn pnpm_dependency_properties_do_not_disable_package_json_defaults() {
    let session = session_with_scoped_property_configs(&[(
        Ecosystem::Npm,
        Some(ManifestKind::PnpmYaml),
        &["customCatalog"],
    )]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "left-pad");
}

#[test]
fn scoped_pnpm_dependency_properties_still_apply_to_pnpm_yaml() {
    let session = session_with_scoped_property_configs(&[(
        Ecosystem::Npm,
        Some(ManifestKind::PnpmYaml),
        &["customCatalog"],
    )]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "\
catalog:
  react: ^18.0.0
customCatalog:
  scheduler: ^0.23.0
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "scheduler");
}
