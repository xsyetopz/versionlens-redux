struct ConfiguredPatternCase<'a> {
    manifest_kind: versionlens_model::ManifestKind,
    pattern: &'a str,
    uri: &'a str,
    language: &'a str,
    fixture: &'a str,
    workspace_root: Option<&'a str>,
    expected_ecosystem: &'a str,
    expected_group: &'a str,
    expected_name: &'a str,
    expected_requirement: &'a str,
}

fn assert_configured_pattern(case: ConfiguredPatternCase<'_>) {
    let session = session_with_file_pattern(crate::FilePatternConfig {
        manifest_kind: case.manifest_kind,
        pattern: case.pattern.to_owned(),
    });
    let output = session.analyze_document(DocumentInput::new(
        case.uri.to_owned(),
        case.language.to_owned(),
        package_file_fixture(case.fixture),
        case.workspace_root.map(str::to_owned),
    ));

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    let dependency = &output.dependencies[0];
    assert_eq!(dependency.ecosystem, case.expected_ecosystem);
    assert_eq!(dependency.group, case.expected_group);
    assert_eq!(dependency.name, case.expected_name);
    assert_eq!(dependency.requirement, case.expected_requirement);
}

#[test]
fn configured_file_pattern_classifies_custom_composer_manifest() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: ComposerJson, pattern: "**/acme.composer.json", uri: "file:///workspace/acme.composer.json", language: "json", fixture: "acme.composer.json", workspace_root: None, expected_ecosystem: "composer", expected_group: "require", expected_name: "acme/package", expected_requirement: "1.2.3" });
}

#[test]
fn configured_file_pattern_supports_brace_alternatives() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: ComposerJson, pattern: "**/{composer.json,acme.composer.json}", uri: "file:///workspace/acme.composer.json", language: "json", fixture: "acme.composer.json", workspace_root: None, expected_ecosystem: "composer", expected_group: "require", expected_name: "acme/package", expected_requirement: "1.2.3" });
}

#[test]
fn configured_file_pattern_supports_workspace_relative_recursive_segments() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: ComposerJson, pattern: "packages/**/acme.composer.json", uri: "file:///workspace/packages/backend/acme.composer.json", language: "json", fixture: "acme.composer.json", workspace_root: Some("/workspace"), expected_ecosystem: "composer", expected_group: "require", expected_name: "acme/package", expected_requirement: "1.2.3" });
}

#[test]
fn configured_file_pattern_supports_character_classes() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: ComposerJson, pattern: "**/acme.composer.jso[n]", uri: "file:///workspace/acme.composer.json", language: "json", fixture: "acme.composer.json", workspace_root: None, expected_ecosystem: "composer", expected_group: "require", expected_name: "acme/package", expected_requirement: "1.2.3" });
}

#[test]
fn configured_file_pattern_supports_character_class_ranges() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: ComposerJson, pattern: "**/acme.composer.jso[m-o]", uri: "file:///workspace/acme.composer.json", language: "json", fixture: "acme.composer.json", workspace_root: None, expected_ecosystem: "composer", expected_group: "require", expected_name: "acme/package", expected_requirement: "1.2.3" });
}

#[test]
fn configured_file_pattern_supports_negated_character_classes() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: ComposerJson, pattern: "**/acme.composer.jso[!x]", uri: "file:///workspace/acme.composer.json", language: "json", fixture: "acme.composer.json", workspace_root: None, expected_ecosystem: "composer", expected_group: "require", expected_name: "acme/package", expected_requirement: "1.2.3" });
}

#[test]
fn configured_file_pattern_supports_micromatch_extglob_alternatives() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: ComposerJson, pattern: "**/@(composer|acme.composer).json", uri: "file:///workspace/acme.composer.json", language: "json", fixture: "acme.composer.json", workspace_root: None, expected_ecosystem: "composer", expected_group: "require", expected_name: "acme/package", expected_requirement: "1.2.3" });
}

#[test]
fn configured_docker_file_pattern_routes_non_yaml_matches_to_dockerfile_parser() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: DockerComposeYaml, pattern: "**/Containerfile", uri: "file:///workspace/Containerfile", language: "plaintext", fixture: "Containerfile", workspace_root: None, expected_ecosystem: "docker", expected_group: "FROM", expected_name: "node", expected_requirement: "20" });
}

#[test]
fn configured_pypi_file_pattern_routes_non_txt_matches_to_toml_parser() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: PythonRequirementsTxt, pattern: "**/pyproject-prod.toml", uri: "file:///workspace/pyproject-prod.toml", language: "toml", fixture: "pyproject-prod.toml", workspace_root: None, expected_ecosystem: "pypi", expected_group: "project.dependencies", expected_name: "requests", expected_requirement: "==2.32.0" });
}

#[test]
fn configured_dub_file_pattern_routes_sdl_matches_to_sdl_parser() {
    assert_configured_pattern(ConfiguredPatternCase { manifest_kind: DubJson, pattern: "**/*.sdl", uri: "file:///workspace/dub.sdl", language: "plaintext", fixture: "dub.sdl", workspace_root: None, expected_ecosystem: "dub", expected_group: "dependencies", expected_name: "vibe-d", expected_requirement: "~>0.9.7" });
}
