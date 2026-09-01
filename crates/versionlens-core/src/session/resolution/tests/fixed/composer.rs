#[test]
fn composer_platform_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture("platform-dependencies-are-fixed.json"), None),
        &[RegistryResponseInput::new("phpunit/phpunit".to_owned(), Composer, r#"{"packages":{"phpunit/phpunit":[{"version":"10.5.0"}]}}"#.to_owned())],
    );

    crate::support::tests::assert_suggestion(&output, 0, "fixed", Some("^8.3"));
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(output.suggestions[1].latest.as_deref(), Some("*"));
    assert_eq!(output.suggestions[2].latest.as_deref(), Some("10.5.0"));
}

#[test]
fn composer_stability_flags_allow_prerelease_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture("stability-flags-allow-prerelease-updates.json"), None),
        &[RegistryResponseInput::new("acme/pkg".to_owned(), Composer, r#"{"packages":{"acme/pkg":[{"version":"1.0.0"},{"version":"1.1.0-beta.1"}]}}"#
                .to_owned())],
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
    let input = DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture(
            "release-resolves-fixed-with-release-update-choices.json",
        ), None);

    let output = resolve_composer(
        &session,
        &input,
        r#"{
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
            }"#,
    );

    let (titles, arguments) = analyze_composer(&session, input);

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "fixed", Some("1.1.1"));
    assert_eq!(
        titles,
        [
            "🟡 fixed 1.1.1",
            "↓  downgrade 1.1.0",
            "↑  patch 1.1.2",
            "↑  minor 1.2.2",
            "↑  latest 2.2.2"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "1.1.0"],
            vec!["updatePatch", "1.1.2"],
            vec!["updateMinor", "1.2.2"],
            vec!["update", "2.2.2"]
        ]
    );
}

#[test]
fn missing_fixed_composer_registry_version_resolves_no_match_with_update_choices() {
    let session = standard_session();
    let input = DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture(
            "missing-fixed-composer-registry-version-resolves-no-match-with-update-choices.json",
        ), None);

    let output = resolve_missing_composer(&session, &input);

    let (titles, arguments) = analyze_composer(&session, input);

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "noMatch", None);
    assert_eq!(
        titles,
        [
            "⚪ no match",
            "↑  patch 0.5.1",
            "↑  minor 0.6.0",
            "↑  latest 1.0.0"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["updatePatch", "0.5.1"],
            vec!["updateMinor", "0.6.0"],
            vec!["update", "1.0.0"]
        ]
    );
}

#[test]
fn invalid_composer_requirement_resolves_invalid_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture(
                "invalid-composer-requirement-resolves-invalid-without-registry-lookup.json",
            ), None),
        &[RegistryResponseInput::new("php-parallel-lint/php-parallel-lint".to_owned(), Composer, r#"{"packages":{"php-parallel-lint/php-parallel-lint":[{"version":"v9.9.9"}]}}"#
                .to_owned())],
    );

    assert_eq!(output.suggestions[0].status, "invalid");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("invalid version")
    );
    assert!(output.edits.is_empty());
}

fn resolve_composer(
    session: &crate::VersionLensSession,
    input: &DocumentInput,
    body: &str,
) -> crate::contract::ResolveDocumentOutput {
    session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "php-parallel-lint/php-parallel-lint".to_owned(),
            Composer,
            body.to_owned(),
        )],
    )
}

fn resolve_missing_composer(
    session: &crate::VersionLensSession,
    input: &DocumentInput,
) -> crate::contract::ResolveDocumentOutput {
    resolve_composer(
        session,
        input,
        r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v0.5.1" },
                  { "version": "v0.6.0" },
                  { "version": "v1.0.0" }
                ]
              }
            }"#,
    )
}

fn analyze_composer(
    session: &crate::VersionLensSession,
    input: DocumentInput,
) -> (Vec<String>, Vec<Vec<String>>) {
    let analysis = session.analyze_document(input);
    let (titles, arguments) =
        crate::session::resolution::tests::lens_titles_and_arguments(&analysis);
    (
        titles.into_iter().map(str::to_owned).collect(),
        arguments
            .into_iter()
            .map(|items| items.into_iter().map(str::to_owned).collect())
            .collect(),
    )
}
