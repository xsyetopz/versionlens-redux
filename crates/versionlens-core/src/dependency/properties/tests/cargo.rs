use super::{DocumentInput, package_file_fixture, session_with_properties};
use versionlens_parsers::Ecosystem::Cargo;

#[test]
fn cargo_target_dependency_properties_match_wildcards() {
    let session = session_with_properties(Cargo, &["target.*.dependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("cargo-target-dependencies.toml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "nix");
}

#[test]
fn dependency_properties_allow_custom_cargo_paths() {
    let session = session_with_properties(Cargo, &["workspace.metadata.versionlens.dependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("cargo-custom-workspace-metadata.toml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "custom");
}
