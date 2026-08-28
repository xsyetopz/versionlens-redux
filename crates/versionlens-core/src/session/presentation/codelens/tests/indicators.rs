#[test]
fn code_lens_title_uses_configured_indicators() {
    let session = standard_session();
    let output = analyze_npm_fixture_with_response(
        &session,
        "left-pad-1.0.0.json",
        r#"{"dist-tags":{"latest":"1.1.0"}}"#,
    );

    let titles = lens_titles(&output);
    let commands = lens_commands(&output);

    assert_eq!(titles, ["M fixed 1.0.0", "U latest 1.1.0"]);
    assert_eq!(commands, ["", "versionlens.suggestion.onUpdateDependency"]);
}

#[test]
fn direct_blank_indicators_use_standard_glyphs_for_status_and_update_lenses() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: SuggestionIndicators {
            latest: "".to_owned(),
            satisfies_latest: " \t\n\u{2003} ".to_owned(),
            directory: "".to_owned(),
            error: "".to_owned(),
            no_match: "".to_owned(),
            matched: "".to_owned(),
            updateable: "".to_owned(),
            updateable_vulnerable: "".to_owned(),
            build: "".to_owned(),
        },
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    assert_eq!(
        session.config.suggestion_indicators,
        crate::standard_suggestion_indicators()
    );
    let output = analyze_npm_fixture_with_response(
        &session,
        "left-pad-1.0.0.json",
        r#"{"dist-tags":{"latest":"1.1.0"}}"#,
    );

    assert_eq!(
        output
            .code_lenses
            .iter()
            .map(|lens| lens.title.as_str())
            .collect::<Vec<_>>(),
        ["🟡 fixed 1.0.0", "↑  latest 1.1.0"]
    );
    assert_eq!(
        output.code_lenses[1].command,
        "versionlens.suggestion.onUpdateDependency"
    );
}

#[test]
fn code_lenses_offer_release_update_choices_for_fixed_versions() {
    let session = standard_session();
    let input = package_document("left-pad-1.0.0.json");

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[npm_versions_response(
            "2.1.0",
            &["1.0.0", "1.0.1", "1.1.0", "2.0.0", "2.1.0"],
        )],
    );

    let titles = lens_titles(&output);
    let arguments = crate::support::tests::code_lens_arguments(&output);

    assert_eq!(
        titles,
        [
            "M fixed 1.0.0",
            "U patch 1.0.1",
            "U minor 1.1.0",
            "U latest 2.1.0"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["updatePatch", "1.0.1"],
            vec!["updateMinor", "1.1.0"],
            vec!["update", "2.1.0"]
        ]
    );
}

#[test]
fn code_lens_ranges_encode_suggestion_order() {
    let session = session_with_indicators(test_indicators(), false);
    let input = package_document("left-pad-1.0.0.json");

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[npm_versions_response(
            "2.1.0",
            &["1.0.0", "1.0.1", "1.1.0", "2.1.0"],
        )],
    );
    let dependency_start = output.dependencies[0].range.start.character;
    let starts = output
        .code_lenses
        .iter()
        .map(|lens| lens.range.start.character)
        .collect::<Vec<_>>();
    let expected_starts = (0..starts.len())
        .map(|order| dependency_start + u32::try_from(order).expect("code lens order fits u32"))
        .collect::<Vec<_>>();
    let zero_width = output
        .code_lenses
        .iter()
        .all(|lens| lens.range.start == lens.range.end);

    assert_eq!(starts, expected_starts);
    assert!(zero_width);
}

#[test]
fn multiline_package_json_code_lenses_stay_on_dependency_lines() {
    let session = session_with_indicators(test_indicators(), false);
    let input = package_document("dev-dependencies.json");

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[
            npm_response("@biomejs/biome", "2.5.2"),
            npm_response("@types/bun", "1.3.14"),
            npm_response("@types/node", "26.0.1"),
            npm_response("@types/vscode", "1.125.0"),
            npm_response("@vscode/vsce", "3.9.2"),
            npm_response("typescript", "6.0.3"),
        ],
    );
    let lenses = output
        .code_lenses
        .iter()
        .map(|lens| {
            (
                lens.range.start.line,
                lens.title.as_str(),
                lens.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        lenses,
        [
            (2, "L latest 2.5.2", ""),
            (3, "L latest 1.3.14", ""),
            (4, "L latest 26.0.1", ""),
            (5, "S satisfies latest 1.125.0", ""),
            (
                5,
                "U latest 1.125.0",
                "versionlens.suggestion.onUpdateDependency"
            ),
            (6, "L latest 3.9.2", ""),
            (7, "L latest 6.0.3", "")
        ]
    );
}
