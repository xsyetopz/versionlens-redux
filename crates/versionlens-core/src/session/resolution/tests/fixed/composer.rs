#[test]
fn composer_platform_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("composer-platform-dependencies-are-fixed.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "phpunit/phpunit".to_owned(),
            ecosystem: Composer,
            body: r#"{"packages":{"phpunit/phpunit":[{"version":"10.5.0"}]}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("^8.3"));
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(output.suggestions[1].latest.as_deref(), Some("*"));
    assert_eq!(output.suggestions[2].latest.as_deref(), Some("10.5.0"));
}

#[test]
fn composer_stability_flags_allow_prerelease_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("composer-stability-flags-allow-prerelease-updates.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "acme/pkg".to_owned(),
            ecosystem: Composer,
            body: r#"{"packages":{"acme/pkg":[{"version":"1.0.0"},{"version":"1.1.0-beta.1"}]}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("1.1.0-beta.1")
    );
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "^1.1.0-beta.1@beta");
}

#[test]
fn fixed_composer_release_resolves_fixed_with_release_update_choices() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "fixed-composer-release-resolves-fixed-with-release-update-choices.json",
        ),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Composer,
            body: r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v1.1.0" },
                  { "version": "v1.1.1" },
                  { "version": "v1.1.2" },
                  { "version": "v1.2.0" },
                  { "version": "v1.2.2" },
                  { "version": "v2.0.0" },
                  { "version": "v2.2.2" }
                ]
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("1.1.1"));
    assert!(output.edits.is_empty());
    assert_eq!(
        titles,
        [
            "🟡 fixed 1.1.1",
            "↑  latest 2.2.2",
            "↑  minor 1.2.2",
            "↑  patch 1.1.2"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "2.2.2"],
            vec!["updateMinor", "1.2.2"],
            vec!["updatePatch", "1.1.2"]
        ]
    );
}

#[test]
fn missing_fixed_composer_registry_version_resolves_no_match_with_update_choices() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///repo/composer.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "missing-fixed-composer-registry-version-resolves-no-match-with-update-choices.json",
        ),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Composer,
            body: r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v0.5.1" },
                  { "version": "v0.6.0" },
                  { "version": "v1.0.0" }
                ]
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());
    assert_eq!(
        titles,
        [
            "⚪ no match",
            "↑  latest 1.0.0",
            "↑  minor 0.6.0",
            "↑  patch 0.5.1"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "1.0.0"],
            vec!["updateMinor", "0.6.0"],
            vec!["updatePatch", "0.5.1"]
        ]
    );
}

#[test]
fn invalid_composer_requirement_resolves_invalid_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "invalid-composer-requirement-resolves-invalid-without-registry-lookup.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "php-parallel-lint/php-parallel-lint".to_owned(),
            ecosystem: Composer,
            body: r#"{"packages":{"php-parallel-lint/php-parallel-lint":[{"version":"v9.9.9"}]}}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "invalid");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("invalid version")
    );
    assert!(output.edits.is_empty());
}
