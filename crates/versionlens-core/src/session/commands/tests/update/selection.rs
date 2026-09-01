#[test]
fn apply_command_uses_code_lens_selector_for_duplicate_names() {
    let session = standard_session();
    let input = DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture(
            "command-uses-code-lens-selector-for-duplicate-names.json",
        ), None);

    let responses = [RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned())];
    session.resolve_document_with_responses(input.clone(), &responses);
    let command_input = input.clone();
    let analyzed = session.analyze_document(input);
    let selector = analyzed
        .code_lenses
        .iter()
        .find(|lens| lens.command == "versionlens.suggestion.onUpdateDependency")
        .and_then(|lens| lens.arguments.get(1))
        .expect("update code lens selector")
        .clone();
    let output = session.apply_command(command_input, None, Some(&selector), &responses);

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "dependencies");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn pyproject_ranges_resolving_latest_do_not_offer_noop_lower_bound_bumps() {
    let session = standard_session();
    let input = DocumentInput::new("file:///pyproject.toml".to_owned(), "toml".to_owned(), package_file_fixture(
            "pyproject-update-code-lenses-advance-lower-bounds-and-preserve-upper-caps.toml",
        ), None);
    let responses = [
        RegistryResponseInput::new("httpx".to_owned(), versionlens_model::Ecosystem::Python, r#"{"info":{"version":"0.28.1"},"releases":{"0.27.0":[],"0.28.1":[{"yanked":false}]}}"#
                .to_owned()),
        RegistryResponseInput::new("httpcore".to_owned(), versionlens_model::Ecosystem::Python, r#"{"info":{"version":"0.28.1"},"releases":{"0.27.0":[],"0.28.1":[{"yanked":false}]}}"#
                .to_owned()),
    ];

    session.resolve_document_with_responses(input.clone(), &responses);
    let analyzed = session.analyze_document(input);

    assert_eq!(
        analyzed
            .code_lenses
            .iter()
            .map(|lens| (lens.title.as_str(), lens.command.as_str()))
            .collect::<Vec<_>>(),
        [
            ("🟢 satisfies latest 0.28.1", ""),
            ("🟢 satisfies latest 0.28.1", "")
        ]
    );
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
        let input = DocumentInput::new("file:///pyproject.toml".to_owned(), "toml".to_owned(), format!("[project]\ndependencies = [\"{package}{requirement}\"]\n"), None);
        let responses = [RegistryResponseInput::new(package.to_owned(), versionlens_model::Ecosystem::Python, format!(
                r#"{{"info":{{"version":"{provider_latest}"}},"releases":{{"{provider_latest}":[{{"yanked":false}}]}}}}"#,
            ))];

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
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("command-updates-only-requested-level.json"), None),
        Some("updateMinor"),
        None,
        &[
            RegistryResponseInput::new("major".to_owned(), Npm, r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned()),
            RegistryResponseInput::new("minor".to_owned(), Npm, r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned()),
            RegistryResponseInput::new("patch".to_owned(), Npm, r#"{"dist-tags":{"latest":"1.0.1"}}"#.to_owned()),
        ],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "minor");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn apply_command_updates_ranged_dependency_to_requested_minor_choice() {
    assert_ranged_choice(
        "command-updates-ranged-dependency-to-requested-minor-choice.json",
        "updateMinor",
        "~1.1.0",
    );
}

#[test]
fn apply_command_updates_ranged_dependency_to_requested_patch_choice() {
    assert_ranged_choice(
        "command-updates-ranged-dependency-to-requested-patch-choice.json",
        "updatePatch",
        "1.0.1",
    );
}

fn assert_ranged_choice(fixture: &str, command: &str, expected: &str) {
    let output = standard_session().apply_command(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture(fixture),
            None,
        ),
        Some(command),
        None,
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"1.0.1":{},"1.1.0":{},"2.0.0":{}}}"#.to_owned(),
        )],
    );
    super::assert_single_named_edit(&output, "left-pad", expected);
}

#[test]
fn apply_command_level_filter_does_not_bump_project_version() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture(
                "command-level-filter-does-not-bump-project-version.json",
            ), None),
        Some("updateMajor"),
        None,
        &[RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned())],
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
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture(
                "command-bulk-update-skips-project-version-edits.json",
            ), None),
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
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture(
                "bulk-update-skips-prerelease-only-invalid-range-updates.json",
            ), None),
        Some("update"),
        None,
        &[RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{
              "dist-tags": { "latest": "5.0.0-beta.1" },
              "versions": {
                "5.0.0-beta.1": {}
              }
            }"#
            .to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "invalidRange");
    assert!(output.edits.is_empty());
}
