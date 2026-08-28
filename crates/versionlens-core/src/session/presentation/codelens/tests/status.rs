#[test]
fn code_lens_title_uses_satisfies_latest_indicator() {
    let session = standard_session();
    let input = package_document("left-pad-caret-1.0.0.json");

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned())],
    );

    assert_eq!(output.code_lenses[0].title, "S satisfies latest 1.1.0");
}

#[test]
fn code_lens_title_uses_latest_indicator_for_current_dependencies() {
    let session = standard_session();
    let input = package_document("typescript-latest.json");

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new("typescript".to_owned(), Npm, r#"{"dist-tags":{"latest":"6.0.3"}}"#.to_owned())],
    );

    assert_eq!(output.code_lenses[0].title, "L latest 6.0.3");
}

#[test]
fn code_lens_title_shows_fixed_git_dependencies() {
    let session = standard_session();
    let input = DocumentInput::new("file:///Cargo.toml".to_owned(), "toml".to_owned(), package_file_fixture("Cargo-git-dependency.toml"), None);

    session.resolve_document(input.clone());
    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[0].title, "M fixed git repository");
}

#[test]
fn missing_suggestion_code_lens_is_omitted_like_upstream() {
    let session = standard_session();
    let input = package_document("left-pad-1.0.0.json");

    let output = session.analyze_document(input);

    assert!(output.code_lenses.is_empty());
}

#[test]
fn code_lens_title_preserves_configured_indicator_spacing_like_non_windows_upstream() {
    let mut indicators = test_indicators();
    indicators.updateable = "U ".to_owned();
    let session = session_with_indicators(indicators, true);
    let output = analyze_npm_fixture_with_response(
        &session,
        "left-pad-1.0.0.json",
        r#"{"dist-tags":{"latest":"1.1.0"}}"#,
    );

    assert_eq!(output.code_lenses[1].title, "U  latest 1.1.0");
}
