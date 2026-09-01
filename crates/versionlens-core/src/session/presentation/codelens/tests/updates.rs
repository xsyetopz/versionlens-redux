#[test]
fn code_lenses_offer_bump_update_choices_for_ranges() {
    let output = build_code_lens_output(
        "left-pad-4.1.0-range.json",
        "5.4.5",
        &[
            "2.1.2", "3.0.0", "3.1.0", "4.0.0", "4.0.1", "4.1.10", "5.1.1", "5.2.0",
            "5.3.3", "5.4.5",
        ],
    );
    let titles = lens_titles(&output);
    let arguments = crate::support::tests::all_code_lens_arguments(&output);

    assert_eq!(
        titles,
        [
            "M satisfies 4.1.10",
            "D downgrade 2.1.2",
            "D downgrade 3.0.0",
            "D downgrade 3.1.0",
            "D downgrade 4.0.0",
            "D downgrade 4.0.1",
            "U bump 4.1.10",
            "U version 5.1.1",
            "U version 5.2.0",
            "U version 5.3.3",
            "U latest 5.4.5",
        ]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "2.1.2"],
            vec!["update", "3.0.0"],
            vec!["update", "3.1.0"],
            vec!["update", "4.0.0"],
            vec!["update", "4.0.1"],
            vec!["update", "4.1.10"],
            vec!["update", "5.1.1"],
            vec!["update", "5.2.0"],
            vec!["update", "5.3.3"],
            vec!["update", "5.4.5"]
        ]
    );
}

#[test]
fn code_lenses_keep_latest_update_choice_for_invalid_ranges() {
    let output = build_code_lens_output(
        "left-pad-invalid-range.json",
        "5.0.0",
        &["1.0.0", "2.0.0", "5.0.0"],
    );
    let titles = lens_titles(&output);
    let arguments = crate::support::tests::all_code_lens_arguments(&output);

    assert_eq!(titles, ["E invalid version range", "U latest 5.0.0"]);
    assert_eq!(arguments, [Vec::<&str>::new(), vec!["update", "5.0.0"]]);
}

#[test]
fn code_lenses_omit_latest_update_choice_for_ranges_satisfying_latest() {
    let output = build_code_lens_output(
        "left-pad-gte-2.json",
        "3.0.0",
        &["1.0.0", "2.0.0", "2.1.0", "3.0.0"],
    );
    let titles = lens_titles(&output);
    let arguments = crate::support::tests::all_code_lens_arguments(&output);

    assert_eq!(
        titles,
        [
            "S satisfies latest 3.0.0",
            "D downgrade 1.0.0",
            "D downgrade 2.1.0"
        ]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "1.0.0"],
            vec!["update", "2.1.0"]
        ]
    );
}

#[test]
fn code_lenses_keep_satisfies_status_for_ranges_with_in_range_updates() {
    let output = build_code_lens_output(
        "left-pad-gte-2-lt-3.json",
        "3.0.0",
        &["1.0.0", "2.0.0", "2.1.0", "3.0.0"],
    );
    let titles = lens_titles(&output);
    let arguments = crate::support::tests::all_code_lens_arguments(&output);

    assert_eq!(
        titles,
        [
            "M satisfies 2.1.0",
            "D downgrade 1.0.0",
            "U bump 2.1.0",
            "U latest 3.0.0"
        ]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "1.0.0"],
            vec!["update", "2.1.0"],
            vec!["update", "3.0.0"]
        ]
    );
}

#[test]
fn github_current_release_omits_noop_latest_and_offers_downgrades() {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///work/.github/workflows/ci.yml".to_owned(),
        "yaml".to_owned(),
        "steps:\n  - uses: actions/checkout@v7.0.1\n".to_owned(),
        None,
    );

    let responses = [RegistryResponseInput::new(
        "actions/checkout".to_owned(),
        versionlens_model::Ecosystem::GitHub,
        r#"[{"name":"v7.0.1"},{"name":"v7.0.0"},{"name":"v6.0.0"}]"#.to_owned(),
    )];
    let resolved = session.resolve_document_with_responses(input.clone(), &responses);
    let output = session.analyze_document(input);

    assert_eq!(
        (
            resolved.suggestions[0].status.as_str(),
            resolved.suggestions[0].latest.as_deref()
        ),
        ("current", Some("7.0.1"))
    );

    assert_eq!(
        lens_titles(&output),
        [
            "L latest v7.0.1",
            "D downgrade v6.0.0",
            "D downgrade v7.0.0"
        ]
    );
    assert_eq!(
        crate::support::tests::all_code_lens_arguments(&output),
        [
            Vec::<&str>::new(),
            vec!["update", "6.0.0"],
            vec!["update", "7.0.0"]
        ]
    );
}

#[test]
fn code_lenses_offer_prerelease_update_choices_by_tag() {
    let session = standard_session();
    let input = package_document("left-pad-prerelease-range.json");

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{
              "versions": {
                "1.0.0-alpha": {},
                "1.0.1-alpha": {},
                "1.2.0-alpha": {},
                "1.2.0-dev": {},
                "1.2.0-beta": {}
              }
            }"#
            .to_owned())],
    );
    let titles = lens_titles(&output);

    assert_eq!(
        titles,
        [
            "U dev 1.2.0-dev",
            "U beta 1.2.0-beta",
            "U alpha 1.2.0-alpha"
        ]
    );
}
