use super::{
    DocumentInput, package_file_fixture, session_with_properties, session_with_property_configs,
    session_with_scoped_property_configs,
};
use versionlens_model::Ecosystem::*;
use versionlens_model::ManifestKind::{NpmPackageJson, PnpmYaml};

fn assert_single_dependency(output: &crate::AnalyzeDocumentOutput, name: &str) {
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, name);
}

#[test]
fn dependency_properties_are_filtered_in_rust() {
    let session = session_with_properties(Npm, &["devDependencies"]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("dev-dependencies.json"),
        None,
    ));

    assert_single_dependency(&output, "is-odd");
}

#[test]
fn dependency_properties_allow_custom_json_paths() {
    let session = session_with_properties(Npm, &["resolutions"]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("resolutions.json"),
        None,
    ));

    assert_single_dependency(&output, "is-even");
}

#[test]
fn dependency_properties_match_wildcard_groups() {
    let session = session_with_properties(Npm, &["packageExtensions.*.peerDependencies"]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///pnpm-workspace.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("package-extensions-peer.yaml"),
        None,
    ));

    assert_single_dependency(&output, "@types/react");
}

#[test]
fn deno_dependency_properties_allow_scopes() {
    let session = session_with_properties(Deno, &["scopes"]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///import_map.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("deno-import-map-scopes.json"),
        None,
    ));

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

    let output = session.analyze_document(DocumentInput::new(
        "file:///deno.json".to_owned(),
        "jsonc".to_owned(),
        package_file_fixture("deno-import-map-scopes.json"),
        None,
    ));

    assert_single_dependency(&output, "@std/assert");
}

#[test]
fn dependency_properties_merge_same_ecosystem_configs() {
    let session = session_with_property_configs(&[
        (Npm, &["devDependencies"][..]),
        (Npm, &["packageExtensions.*.peerDependencies"][..]),
    ]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///pnpm-workspace.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("package-extensions-scheduler.yaml"),
        None,
    ));

    assert_single_dependency(&output, "scheduler");
}

#[test]
fn dependency_properties_allow_custom_pnpm_paths() {
    let session = session_with_properties(Npm, &["customCatalog"]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///pnpm-workspace.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("custom-catalog.yaml"),
        None,
    ));

    assert_single_dependency(&output, "scheduler");
}

#[test]
fn scoped_npm_dependency_properties_apply_to_package_json5() {
    let session = session_with_scoped_property_configs(&[(
        Npm,
        Some(NpmPackageJson),
        &["devDependencies"][..],
    )]);
    let input = DocumentInput::new(
        "file:///package.json5".to_owned(),
        "json5".to_owned(),
        package_file_fixture("package-dev-dependencies.json5"),
        None,
    );

    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "typescript");
    assert_eq!(dependencies[0].group, "devDependencies");
}

#[test]
fn scoped_npm_dependency_properties_apply_to_package_yaml() {
    let session =
        session_with_scoped_property_configs(&[(Npm, Some(NpmPackageJson), &["devDependencies"])]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///package.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("package-dev-dependencies.yaml"),
        None,
    ));

    assert_single_dependency(&output, "typescript");
}

#[test]
fn npm_dependency_properties_do_not_disable_pnpm_yaml_defaults() {
    let session =
        session_with_scoped_property_configs(&[(Npm, Some(NpmPackageJson), &["devDependencies"])]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///pnpm-workspace.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("catalog.yaml"),
        None,
    ));

    assert_single_dependency(&output, "react");
}

#[test]
fn pnpm_dependency_properties_do_not_disable_package_json_defaults() {
    let session =
        session_with_scoped_property_configs(&[(Npm, Some(PnpmYaml), &["customCatalog"])]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("dependencies.json"),
        None,
    ));

    assert_single_dependency(&output, "left-pad");
}

#[test]
fn scoped_pnpm_dependency_properties_still_apply_to_pnpm_yaml() {
    let session =
        session_with_scoped_property_configs(&[(Npm, Some(PnpmYaml), &["customCatalog"])]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///pnpm-workspace.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("custom-catalog.yaml"),
        None,
    ));

    assert_single_dependency(&output, "scheduler");
}
