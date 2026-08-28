#[test]
fn apply_command_sorts_requirements_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///requirements.txt", "pip-requirements", "requirements-unsorted-with-comment.txt");

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
        DocumentInput::new("file:///requirements.txt".to_owned(), "pip-requirements".to_owned(), text.clone(), None),
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
    assert_sorted_pyproject_fixture(
        "pyproject-project-unsorted.toml",
        "[project]\ndependencies = [\n  \"alpha==1\",\n  \"zeta==1\"\n]",
    );
}

#[test]
fn apply_command_sorts_pyproject_poetry_dependencies() {
    assert_sorted_pyproject_fixture(
        "pyproject-poetry-unsorted.toml",
        "[tool.poetry.dependencies]\nalpha = \"1\"\nzeta = \"1\"",
    );
}

fn assert_sorted_pyproject_fixture(fixture: &str, expected: &str) {
    let text = package_file_fixture(fixture);
    let output = standard_session().apply_command(
        DocumentInput::new(
            "file:///pyproject.toml".to_owned(),
            "toml".to_owned(),
            text.clone(),
            None,
        ),
        Some("sort"),
        None,
        &[],
    );
    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(apply_line_edits(&text, &output.edits), expected);
}

#[test]
fn apply_command_sorts_pub_dependencies_by_group() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///pubspec.yaml", "yaml", "groups-unsorted.yaml");

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

    let output = sort_fixture(&session, "file:///pubspec.yaml", "yaml", "blank-version.yaml");

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "  equatable:");
    assert_eq!(output.edits[1].new_text, "  flutter_bloc: 0.10.1");
}

#[test]
fn apply_command_sorts_complex_pub_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///pubspec.yaml", "yaml", "complex.yaml");

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

    let output = sort_fixture(&session, "file:///package.json", "json", "groups-unsorted.json");

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

    let output = sort_fixture(&session, "file:///package.json", "json", "metadata-unsorted.json");

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "    \"alpha\": \"1\",");
    assert_eq!(output.edits[1].new_text, "    \"zeta\": \"1\"");
}

#[test]
fn apply_command_does_not_sort_docker_compose_images() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///docker-compose.yaml", "yaml", "docker-compose-images.yaml");

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_sorts_composer_require_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///composer.json", "json", "composer-require-unsorted.json");

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

    let output = sort_fixture(
        &session,
        "file:///deno.json",
        "jsonc",
        "deno-scopes-unsorted.json",
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

    let output = sort_fixture(&session, "file:///pnpm-workspace.yaml", "yaml", "pnpm-workspace-named-catalog-unsorted.yaml");

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "    react: ^18.3.1");
    assert_eq!(output.edits[1].new_text, "    react-dom: ^19.2.7");
}

#[test]
fn apply_command_sorts_package_json_named_workspace_catalog_dependencies() {
    let session = session_with_dependency_properties(Npm, &["workspaces.catalogs.*"]);

    let output = sort_fixture(&session, "file:///package.json", "json", "workspace-catalog-unsorted.json");

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "        \"react\": \"^18.3.1\",");
    assert_eq!(
        output.edits[1].new_text,
        "        \"react-dom\": \"^19.2.7\""
    );
}
