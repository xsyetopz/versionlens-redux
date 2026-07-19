use versionlens_model::Ecosystem::{Cargo, Composer, Deno, Dotnet, Go, Maven, Python};
#[test]
fn apply_command_preserves_semver_requirement_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-preserves-semver-requirement-prefix.json"),
            workspace_root: None,
        },
        Some("update"),
        Some("left-pad"),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "^2.0.0");
}

#[test]
fn apply_command_preserves_composer_stability_flag_suffix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-preserves-composer-stability-flag-suffix.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("monolog/monolog"),
        &[RegistryResponseInput {
            package: "monolog/monolog".to_owned(),
            ecosystem: Composer,
            body: r#"{"packages":{"monolog/monolog":[{"version":"1.1.0"}]}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0@beta");
}

#[test]
fn apply_command_updates_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-project-version-by-requested-level.json",
            ),
            workspace_root: None,
        },
        Some("updateMajor"),
        Some("1.2.3"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

#[test]
fn apply_command_updates_jsr_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///jsr.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-jsr-project-version-by-requested-level.json",
            ),
            workspace_root: None,
        },
        Some("updatePatch"),
        Some("@scope/pkg"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.suggestions[0].dependency.name, "@scope/pkg");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn apply_command_updates_deno_json_jsr_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-deno-json-jsr-project-version-by-requested-level.json",
            ),
            workspace_root: None,
        },
        Some("updatePatch"),
        Some("@scope/pkg"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.suggestions[0].dependency.name, "@scope/pkg");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn apply_command_updates_prerelease_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-prerelease-project-version-by-requested-level.json",
            ),
            workspace_root: None,
        },
        Some("updateRelease"),
        Some("1.2.3-beta.4"),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.3");
}

#[test]
fn apply_command_updates_only_project_versions_for_prerelease_command() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-only-project-versions-for-prerelease-command.json",
            ),
            workspace_root: None,
        },
        Some("updatePrerelease"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.3-beta.5");
}

#[test]
fn apply_command_updates_cargo_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-cargo-project-version-by-requested-level.toml",
            ),
            workspace_root: None,
        },
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
        DocumentInput {
            uri: "file:///Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-cargo-renamed-package-version-preserving-alias.toml",
            ),
            workspace_root: None,
        },
        None,
        Some("local_name"),
        &[RegistryResponseInput {
            package: "registry-name".to_owned(),
            ecosystem: Cargo,
            body: r#"{"versions":[{"num":"1.1.0"}]}"#.to_owned(),
        }],
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
        input: DocumentInput {
            uri: "file:///go.mod".to_owned(),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-go-hyphenated-prerelease-version.mod",
            ),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("example.test/prerelease"),
        selected_version: Some("v1.0.0"),
        responses: &[RegistryResponseInput {
            package: "example.test/prerelease".to_owned(),
            ecosystem: Go,
            body: "v1.0.0-alpha-beta\nv1.0.0\n".to_owned(),
        }],
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
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-bare-requirements-with-equals-prefix.txt",
            ),
            workspace_root: None,
        },
        None,
        Some("importlib-metadata"),
        &[RegistryResponseInput {
            package: "importlib-metadata".to_owned(),
            ecosystem: Python,
            body: r#"{"info":{"version":"8.7.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "==8.7.0");
}

#[test]
fn apply_command_updates_empty_pipfile_requirements_with_equals_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Pipfile".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-empty-pipfile-requirements-with-equals-prefix.Pipfile",
            ),
            workspace_root: None,
        },
        None,
        Some("magic"),
        &[RegistryResponseInput {
            package: "magic".to_owned(),
            ecosystem: Python,
            body: r#"{"info":{"version":"1.2.3"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "==1.2.3");
}

#[test]
fn apply_command_inserts_missing_deno_import_versions() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: package_file_fixture("apply-command-inserts-missing-deno-import-versions.json"),
            workspace_root: None,
        },
        None,
        Some("@std/assert"),
        &[RegistryResponseInput {
            package: "@std/assert".to_owned(),
            ecosystem: Deno,
            body: r#"{"versions":{"1.0.1":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "jsr:@std/assert@1.0.1");
}
