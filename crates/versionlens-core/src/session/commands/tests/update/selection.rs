#[test]
fn apply_command_uses_code_lens_selector_for_duplicate_names() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "apply-command-uses-code-lens-selector-for-duplicate-names.json",
        ),
        workspace_root: None,
    };

    let responses = [RegistryResponseInput {
        package: "left-pad".to_owned(),
        ecosystem: Npm,
        body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
    }];
    session.resolve_document_with_responses(input.clone(), &responses);
    let analyzed = session.analyze_document(input.clone());
    let selector = analyzed
        .code_lenses
        .iter()
        .find(|lens| lens.command == "versionlens.suggestion.onUpdateDependency")
        .and_then(|lens| lens.arguments.get(1))
        .expect("update code lens selector")
        .clone();
    let output = session.apply_command(input, None, Some(&selector), &responses);

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "dependencies");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn pyproject_update_code_lenses_advance_lower_bounds_and_preserve_upper_caps() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "pyproject-update-code-lenses-advance-lower-bounds-and-preserve-upper-caps.toml",
        ),
        workspace_root: None,
    };
    let responses = [
        RegistryResponseInput {
            package: "httpx".to_owned(),
            ecosystem: versionlens_model::Ecosystem::Python,
            body: r#"{"info":{"version":"0.28.1"},"releases":{"0.27.0":[],"0.28.1":[{"yanked":false}]}}"#
                .to_owned(),
        },
        RegistryResponseInput {
            package: "httpcore".to_owned(),
            ecosystem: versionlens_model::Ecosystem::Python,
            body: r#"{"info":{"version":"0.28.1"},"releases":{"0.27.0":[],"0.28.1":[{"yanked":false}]}}"#
                .to_owned(),
        },
    ];

    session.resolve_document_with_responses(input.clone(), &responses);
    let analyzed = session.analyze_document(input.clone());

    for (package, group) in [
        ("httpx", "project.dependencies"),
        ("httpcore", "project.optional-dependencies.test"),
    ] {
        let arguments = analyzed
            .code_lenses
            .iter()
            .find(|lens| {
                lens.command == "versionlens.suggestion.onUpdateDependency"
                    && lens.arguments.first().is_some_and(|name| name == package)
            })
            .map(|lens| lens.arguments.as_slice())
            .expect("Python update code lens arguments");
        let output = session.apply_command_with_selected_version(ApplyCommandRequest {
            input: input.clone(),
            command: arguments.get(2).map(String::as_str),
            dependency_name: arguments.get(1).map(String::as_str),
            selected_version: arguments.get(3).map(String::as_str),
            responses: &responses,
        });

        assert_eq!(output.suggestions.len(), 1);
        assert_eq!(output.suggestions[0].dependency.group, group);
        assert_eq!(output.edits.len(), 1);
        assert_eq!(
            output.edits[0].range,
            output.suggestions[0].dependency.requirement_range
        );
        assert_eq!(output.edits[0].new_text, ">=0.28.1, <1");
    }
}

#[test]
fn pyproject_selected_pep440_updates_preserve_or_repair_extended_bounds() {
    for (package, requirement, provider_latest, selected, expected) in [
        (
            "epoch-package",
            ">=1!1.0,<1!2.0",
            "1!1.5",
            "1!1.5",
            ">=1!1.5, <1!2.0",
        ),
        (
            "post-package",
            ">=1.0.post1,<1.0.post3",
            "1.0.post2",
            "1.0.post2",
            ">=1.0.post2, <1.0.post3",
        ),
        (
            "dev-package",
            ">=1.0.dev1,<1.0",
            "1.0",
            "1.0.dev2",
            ">=1.0.dev2, <=1.0.dev2",
        ),
        (
            "local-package",
            ">=1.0,<2.0,!=1.5+linux",
            "1.5+mac",
            "1.5+mac",
            ">=1.5, <2.0, !=1.5+linux",
        ),
    ] {
        let session = standard_session();
        let input = DocumentInput {
            uri: "file:///pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: format!("[project]\ndependencies = [\"{package}{requirement}\"]\n"),
            workspace_root: None,
        };
        let responses = [RegistryResponseInput {
            package: package.to_owned(),
            ecosystem: versionlens_model::Ecosystem::Python,
            body: format!(
                r#"{{"info":{{"version":"{provider_latest}"}},"releases":{{"{provider_latest}":[{{"yanked":false}}]}}}}"#,
            ),
        }];

        let output = session.apply_command_with_selected_version(ApplyCommandRequest {
            input,
            command: Some("update"),
            dependency_name: Some(package),
            selected_version: Some(selected),
            responses: &responses,
        });

        assert_eq!(output.suggestions.len(), 1, "{package}");
        assert_eq!(
            output.suggestions[0].dependency.requirement, requirement,
            "{package}",
        );
        assert_eq!(output.edits.len(), 1, "{package}");
        assert_eq!(output.edits[0].new_text, expected, "{package}");
    }
}

#[test]
fn apply_command_updates_only_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-only-requested-level.json"),
            workspace_root: None,
        },
        Some("updateMinor"),
        None,
        &[
            RegistryResponseInput {
                package: "major".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "minor".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            },
            RegistryResponseInput {
                package: "patch".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"1.0.1"}}"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "minor");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn apply_command_updates_ranged_dependency_to_requested_minor_choice() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-ranged-dependency-to-requested-minor-choice.json"),
            workspace_root: None,
        },
        Some("updateMinor"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"1.0.1":{},"1.1.0":{},"2.0.0":{}}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "left-pad");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "~1.1.0");
}

#[test]
fn apply_command_updates_ranged_dependency_to_requested_patch_choice() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-ranged-dependency-to-requested-patch-choice.json"),
            workspace_root: None,
        },
        Some("updatePatch"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"1.0.1":{},"1.1.0":{},"2.0.0":{}}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "left-pad");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.0.1");
}

#[test]
fn apply_command_level_filter_does_not_bump_project_version() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-level-filter-does-not-bump-project-version.json",
            ),
            workspace_root: None,
        },
        Some("updateMajor"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "left-pad");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

#[test]
fn apply_command_bulk_update_skips_project_version_edits() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-bulk-update-skips-project-version-edits.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn bulk_update_skips_prerelease_only_invalid_range_updates() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "bulk-update-skips-prerelease-only-invalid-range-updates.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{
              "dist-tags": { "latest": "5.0.0-beta.1" },
              "versions": {
                "5.0.0-beta.1": {}
              }
            }"#
            .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "invalidRange");
    assert!(output.edits.is_empty());
}
