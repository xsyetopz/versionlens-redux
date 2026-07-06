use super::{
    DocumentInput, package_file_fixture, session_with_properties, session_with_property_configs,
    session_with_scoped_property_configs,
};
use versionlens_parsers::Ecosystem::{Deno, Npm};
use versionlens_parsers::ManifestKind::{NpmPackageJson, PnpmYaml};

#[test]
fn dependency_properties_are_filtered_in_rust() {
    let session = session_with_properties(Npm, &["devDependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-dev-dependencies.json"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "is-odd");
}

#[test]
fn dependency_properties_allow_custom_json_paths() {
    let session = session_with_properties(Npm, &["resolutions"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-resolutions.json"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "is-even");
}

#[test]
fn dependency_properties_match_wildcard_groups() {
    let session = session_with_properties(Npm, &["packageExtensions.*.peerDependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pnpm-package-extensions-peer.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "@types/react");
}

#[test]
fn deno_dependency_properties_allow_scopes() {
    let session = session_with_properties(Deno, &["scopes"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///import_map.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("deno-import-map-scopes.json"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(
        output.dependencies[0].group,
        "scopes.https://deno.land/x/app/"
    );
    assert_eq!(output.dependencies[0].name, "@scope/pkg");
}

#[test]
fn deno_dependency_properties_filter_before_extraction() {
    let session = session_with_properties(Deno, &["imports"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: package_file_fixture("deno-import-map-scopes.json"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "@std/assert");
}

#[test]
fn dependency_properties_merge_same_ecosystem_configs() {
    let session = session_with_property_configs(&[
        (Npm, &["devDependencies"][..]),
        (Npm, &["packageExtensions.*.peerDependencies"][..]),
    ]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pnpm-package-extensions-scheduler.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "scheduler");
}

#[test]
fn dependency_properties_allow_custom_pnpm_paths() {
    let session = session_with_properties(Npm, &["customCatalog"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pnpm-custom-catalog.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "scheduler");
}

#[test]
fn scoped_npm_dependency_properties_apply_to_package_json5() {
    let session = session_with_scoped_property_configs(&[(
        Npm,
        Some(NpmPackageJson),
        &["devDependencies"][..],
    )]);
    let input = DocumentInput {
        uri: "file:///package.json5".to_owned(),
        language_id: "json5".to_owned(),
        text: package_file_fixture("package-dev-dependencies.json5"),
        workspace_root: None,
    };

    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "typescript");
    assert_eq!(dependencies[0].group, "devDependencies");
}

#[test]
fn scoped_npm_dependency_properties_apply_to_package_yaml() {
    let session =
        session_with_scoped_property_configs(&[(Npm, Some(NpmPackageJson), &["devDependencies"])]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("package-dev-dependencies.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "typescript");
}

#[test]
fn npm_dependency_properties_do_not_disable_pnpm_yaml_defaults() {
    let session =
        session_with_scoped_property_configs(&[(Npm, Some(NpmPackageJson), &["devDependencies"])]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pnpm-catalog.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "react");
}

#[test]
fn pnpm_dependency_properties_do_not_disable_package_json_defaults() {
    let session =
        session_with_scoped_property_configs(&[(Npm, Some(PnpmYaml), &["customCatalog"])]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("package.json"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "left-pad");
}

#[test]
fn scoped_pnpm_dependency_properties_still_apply_to_pnpm_yaml() {
    let session =
        session_with_scoped_property_configs(&[(Npm, Some(PnpmYaml), &["customCatalog"])]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pnpm-custom-catalog.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "scheduler");
}
