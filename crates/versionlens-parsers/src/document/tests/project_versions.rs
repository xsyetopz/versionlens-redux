#[test]
fn project_version_lookup_reads_npm_and_cargo_version_fields_with_exact_ranges() {
    let npm = r#"{"name":"demo","version":"0.4.4"}"#;
    let npm_versions = parse_project_versions(&DocumentInput::new(
        "file:///work/package.json".to_owned(),
        "json".to_owned(),
        npm.to_owned(),
        None,
    ));
    assert_eq!(npm_versions.len(), 1);
    assert_eq!(npm_versions[0].group, "version");
    assert_eq!(npm_versions[0].requirement, "0.4.4");
    assert_eq!(
        extract_range(npm, npm_versions[0].requirement_range),
        "0.4.4"
    );

    let cargo = "[package]\nname = \"demo\"\nversion = \"1.2.3\"\n\n[workspace.package]\nversion = \"2.0.0\"\n";
    let cargo_versions = parse_project_versions(&DocumentInput::new(
        "file:///work/Cargo.toml".to_owned(),
        "toml".to_owned(),
        cargo.to_owned(),
        None,
    ));
    assert_eq!(cargo_versions.len(), 2);
    assert_eq!(cargo_versions[0].group, "package");
    assert_eq!(cargo_versions[0].name, "version");
    assert_eq!(cargo_versions[0].requirement, "1.2.3");
    assert_eq!(
        extract_range(cargo, cargo_versions[0].requirement_range),
        "1.2.3"
    );
    assert_eq!(cargo_versions[1].requirement, "2.0.0");
    assert_eq!(
        extract_range(cargo, cargo_versions[1].requirement_range),
        "2.0.0"
    );
}

#[test]
fn project_version_lookup_reads_inherited_cargo_version_as_a_workspace_reference() {
    let cargo = "[package]\nname = \"member\"\nversion.workspace = true\n";
    let versions = parse_project_versions(&DocumentInput::new(
        "file:///work/member/Cargo.toml".to_owned(),
        "toml".to_owned(),
        cargo.to_owned(),
        None,
    ));
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].requirement, "workspace:true");
    assert_eq!(extract_range(cargo, versions[0].requirement_range), "true");
}

#[test]
fn project_version_lookup_takes_the_manifest_kind_for_non_file_uris() {
    let text = r#"{"name":"demo","version":"1.0.0"}"#;
    let input = DocumentInput::new(
        "package.json".to_owned(),
        "json".to_owned(),
        text.to_owned(),
        None,
    );
    let versions = parse_project_versions_as_manifest(&input, ManifestKind::NpmPackageJson);
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].requirement, "1.0.0");
}
