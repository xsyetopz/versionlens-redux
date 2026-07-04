use super::{DocumentInput, Ecosystem, session_with_properties};

#[test]
fn cargo_target_dependency_properties_match_wildcards() {
    let session = session_with_properties(Ecosystem::Cargo, &["target.*.dependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: "\
[dependencies]
serde = \"1\"

[target.'cfg(unix)'.dependencies]
nix = \"0.29\"
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "nix");
}

#[test]
fn dependency_properties_allow_custom_cargo_paths() {
    let session = session_with_properties(
        Ecosystem::Cargo,
        &["workspace.metadata.versionlens.dependencies"],
    );

    let output = session.analyze_document(DocumentInput {
        uri: "file:///Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: "\
[dependencies]
serde = \"1\"

[workspace.metadata.versionlens.dependencies]
custom = \"1\"
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "custom");
}
