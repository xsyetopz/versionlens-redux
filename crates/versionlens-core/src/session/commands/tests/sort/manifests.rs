#[test]
fn apply_command_sorts_requirements_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: package_file_fixture("requirements-unsorted-with-comment.txt"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "alpha==1");
    assert_eq!(output.edits[1].new_text, "zeta==1");
}

#[test]
fn apply_command_sorts_smoke_requirements_dependencies() {
    let session = standard_session();

    let text = package_file_fixture("requirements-smoke.txt");
    let output = session.apply_command(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: text.clone(),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(
        apply_line_edits(&text, &output.edits),
        "# Requirements for smoke testing\ndjango<=3.2\nflask>=2.0\nnot_found_package==1.17.0\nnumpy<1.22 # this should not cause issues\npandas~=1.2\npytest>3.0\npython-dateutil\nrequests==2.25.1\nsix==1.17.0\nurllib3===1.26.5"
    );
}

#[test]
fn apply_command_sorts_pyproject_project_dependencies() {
    let session = standard_session();

    let text = package_file_fixture("pyproject-project-unsorted.toml");
    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.clone(),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        apply_line_edits(&text, &output.edits),
        "[project]\ndependencies = [\n  \"alpha==1\",\n  \"zeta==1\"\n]"
    );
}

#[test]
fn apply_command_sorts_pyproject_poetry_dependencies() {
    let session = standard_session();

    let text = package_file_fixture("pyproject-poetry-unsorted.toml");
    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.clone(),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        apply_line_edits(&text, &output.edits),
        "[tool.poetry.dependencies]\nalpha = \"1\"\nzeta = \"1\""
    );
}

#[test]
fn apply_command_sorts_pub_dependencies_by_group() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pubspec-groups-unsorted.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 4);
    assert_eq!(output.edits[0].new_text, "  alpha: 1");
    assert_eq!(output.edits[1].new_text, "  zeta: 1");
    assert_eq!(output.edits[2].new_text, "  a-dev: 1");
    assert_eq!(output.edits[3].new_text, "  z-dev: 1");
}

#[test]
fn apply_command_sorts_pub_dependencies_with_blank_versions() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pubspec-blank-version.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "  equatable:");
    assert_eq!(output.edits[1].new_text, "  flutter_bloc: 0.10.1");
}

#[test]
fn apply_command_sorts_complex_pub_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pubspec-complex.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "  equatable: ^0.2.0");
    assert_eq!(
        output.edits[1].new_text,
        "  sqflite:\n    git:\n      url: https://github.com/tekartik/sqflite\n      path: sqflite"
    );
}

#[test]
fn apply_command_sorts_package_json_dependencies_by_group() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("package-groups-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 4);
    assert_eq!(output.edits[0].new_text, "    \"alpha\": \"1\",");
    assert_eq!(output.edits[1].new_text, "    \"zeta\": \"1\"");
    assert_eq!(output.edits[2].new_text, "    \"a-dev\": \"1\",");
    assert_eq!(output.edits[3].new_text, "    \"z-dev\": \"1\"");
}

#[test]
fn apply_command_sorts_package_json_dependencies_with_metadata_entries() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("package-metadata-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "    \"alpha\": \"1\",");
    assert_eq!(output.edits[1].new_text, "    \"zeta\": \"1\"");
}

#[test]
fn apply_command_does_not_sort_docker_compose_images() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///docker-compose.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("docker-compose-images.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_sorts_composer_require_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("composer-require-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        output.edits[0].new_text,
        "    \"allocine/twigcs\": \"^3.1.3\","
    );
    assert_eq!(
        output.edits[1].new_text,
        "    \"symfony/console\": \"8.1.*\""
    );
}

#[test]
fn apply_command_sorts_deno_scoped_imports_within_each_scope() {
    let session = session_with_dependency_properties(Deno, &["scopes"]);

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: package_file_fixture("deno-scopes-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 4);
    assert_eq!(
        output.edits[0].new_text,
        "      \"chalk\": \"npm:chalk@5.3.0\","
    );
    assert_eq!(
        output.edits[1].new_text,
        "      \"zeta\": \"npm:zeta@1.0.0\""
    );
    assert_eq!(
        output.edits[2].new_text,
        "      \"alpha\": \"jsr:@scope/alpha@1.0.0\","
    );
    assert_eq!(
        output.edits[3].new_text,
        "      \"bravo\": \"jsr:@scope/bravo@1.0.0\""
    );
}

#[test]
fn apply_command_sorts_pnpm_named_catalog_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pnpm-workspace.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pnpm-workspace-named-catalog-unsorted.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "    react: ^18.3.1");
    assert_eq!(output.edits[1].new_text, "    react-dom: ^19.2.7");
}

#[test]
fn apply_command_sorts_package_json_named_workspace_catalog_dependencies() {
    let session = session_with_dependency_properties(Npm, &["workspaces.catalogs.*"]);

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("package-workspace-catalog-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "        \"react\": \"^18.3.1\",");
    assert_eq!(
        output.edits[1].new_text,
        "        \"react-dom\": \"^19.2.7\""
    );
}
