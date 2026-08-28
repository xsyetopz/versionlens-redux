#[test]
fn apply_command_preserves_semver_requirement_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("command-preserves-semver-requirement-prefix.json"), None),
        Some("update"),
        Some("left-pad"),
        &[RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned())],
    );

    assert_single_edit(&output, "^2.0.0");
}

#[test]
fn apply_command_preserves_composer_stability_flag_suffix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///composer.json".to_owned(), "json".to_owned(), package_file_fixture(
                "command-preserves-composer-stability-flag-suffix.json",
            ), None),
        Some("update"),
        Some("monolog/monolog"),
        &[RegistryResponseInput::new("monolog/monolog".to_owned(), Composer, r#"{"packages":{"monolog/monolog":[{"version":"1.1.0"}]}}"#.to_owned())],
    );

    assert_single_edit(&output, "1.1.0@beta");
}

#[test]
fn apply_command_updates_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture(
                "command-updates-project-version-by-requested-level.json",
            ), None),
        Some("updateMajor"),
        Some("1.2.3"),
        &[],
    );

    assert_project_edit(&output, "2.0.0");
}

#[test]
fn apply_command_updates_jsr_project_version_by_requested_level() {
    assert_project_version_update(
        "file:///jsr.json",
        "json",
        "command-updates-jsr-project-version-by-requested-level.json",
        "@scope/pkg",
    );
}

#[test]
fn apply_command_updates_deno_json_jsr_project_version_by_requested_level() {
    assert_project_version_update(
        "file:///deno.json",
        "jsonc",
        "command-updates-deno-json-jsr-project-version-by-requested-level.json",
        "@scope/pkg",
    );
}

fn assert_project_version_update(uri: &str, language_id: &str, fixture: &str, name: &str) {
    let output = standard_session().apply_command(
        DocumentInput::new(
            uri.to_owned(),
            language_id.to_owned(),
            package_file_fixture(fixture),
            None,
        ),
        Some("updatePatch"),
        Some(name),
        &[],
    );
    assert_project_update(&output, name, "1.2.4");
}

#[test]
fn apply_command_updates_prerelease_project_version_by_requested_level() {
    assert_prerelease_project_version_update(
        "command-updates-prerelease-project-version-by-requested-level.json",
        Some("updateRelease"),
        Some("1.2.3-beta.4"),
        "1.2.3",
    );
}

#[test]
fn apply_command_updates_only_project_versions_for_prerelease_command() {
    assert_prerelease_project_version_update(
        "command-updates-only-project-versions-for-prerelease-command.json",
        Some("updatePrerelease"),
        None,
        "1.2.3-beta.5",
    );
}

fn assert_prerelease_project_version_update(
    fixture: &str,
    command: Option<&str>,
    selected_version: Option<&str>,
    expected: &str,
) {
    let output = standard_session().apply_command(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture(fixture),
            None,
        ),
        command,
        selected_version,
        &[crate::support::tests::npm_latest_response("left-pad", "2.0.0")],
    );
    assert_project_edit(&output, expected);
}

#[test]
fn apply_command_updates_cargo_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///Cargo.toml".to_owned(), "toml".to_owned(), package_file_fixture(
                "command-updates-cargo-project-version-by-requested-level.toml",
            ), None),
        Some("updatePatch"),
        Some("version"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "package");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn apply_command_updates_cargo_renamed_package_version_preserving_alias() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///Cargo.toml".to_owned(), "toml".to_owned(), package_file_fixture(
                "command-updates-cargo-renamed-package-version-preserving-alias.toml",
            ), None),
        None,
        Some("local_name"),
        &[RegistryResponseInput::new("registry-name".to_owned(), Cargo, r#"{"versions":[{"num":"1.1.0"}]}"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "local_name");
    assert_eq!(
        output.suggestions[0].dependency.hosted_name.as_deref(),
        Some("registry-name")
    );
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn apply_command_updates_go_hyphenated_prerelease_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///go.mod".to_owned(), "go.mod".to_owned(), package_file_fixture(
                "command-updates-go-hyphenated-prerelease-version.mod",
            ), None),
        command: Some("update"),
        dependency_name: Some("example.test/prerelease"),
        selected_version: Some("v1.0.0"),
        responses: &[RegistryResponseInput::new("example.test/prerelease".to_owned(), Go, "v1.0.0-alpha-beta\nv1.0.0\n".to_owned())],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(
        output.suggestions[0].dependency.requirement,
        "v1.0.0-alpha-beta"
    );
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "v1.0.0");
}

#[test]
fn apply_command_updates_bare_requirements_with_equals_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///requirements.txt".to_owned(), "pip-requirements".to_owned(), package_file_fixture(
                "command-updates-bare-requirements-with-equals-prefix.txt",
            ), None),
        None,
        Some("importlib-metadata"),
        &[RegistryResponseInput::new("importlib-metadata".to_owned(), Python, r#"{"info":{"version":"8.7.0"}}"#.to_owned())],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "==8.7.0");
}

#[test]
fn apply_command_updates_empty_pipfile_requirements_with_equals_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///Pipfile".to_owned(), "toml".to_owned(), package_file_fixture(
                "command-updates-empty-pipfile-requirements-with-equals-prefix.Pipfile",
            ), None),
        None,
        Some("magic"),
        &[RegistryResponseInput::new("magic".to_owned(), Python, r#"{"info":{"version":"1.2.3"}}"#.to_owned())],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "==1.2.3");
}

#[test]
fn apply_command_inserts_missing_deno_import_versions() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///deno.json".to_owned(), "jsonc".to_owned(), package_file_fixture("command-inserts-missing-deno-import-versions.json"), None),
        None,
        Some("@std/assert"),
        &[RegistryResponseInput::new("@std/assert".to_owned(), Deno, r#"{"versions":{"1.0.1":{}}}"#.to_owned())],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "jsr:@std/assert@1.0.1");
}
