use super::*;

fn vulnerable_output(
    session: VersionLensSession,
    fixture: &str,
    response: RegistryResponseInput,
) -> AnalyzeDocumentOutput {
    let input = package_document(fixture);
    session.resolve_document_with_responses(input.clone(), &[response]);
    session.analyze_document(input)
}

#[test]
fn code_lens_title_marks_vulnerable_update_targets() {
    let output = vulnerable_output(
        standard_session(),
        "left-pad-1.0.0.json",
        npm_vulnerability_response(
            Some("1.1.0"),
            &[],
            "OSV-1",
            "target issue",
            Some(("1.1.0", "2.0.0")),
        ),
    );

    assert_eq!(output.code_lenses[1].title, "V latest 1.1.0");
}

#[test]
fn code_lens_title_does_not_mark_update_that_fixes_current_vulnerability() {
    let output = vulnerable_output(
        standard_session(),
        "left-pad-1.0.0.json",
        npm_vulnerability_response(
            Some("1.1.0"),
            &[],
            "OSV-1",
            "current issue",
            Some(("0", "1.1.0")),
        ),
    );

    assert_eq!(output.code_lenses[1].title, "U latest 1.1.0");
}

#[test]
fn vulnerable_update_indicator_falls_back_to_warning_when_configured_indicator_is_empty() {
    let output = vulnerable_output(
        session_with_empty_vulnerable_indicator(),
        "left-pad-1.1.1.json",
        npm_vulnerability_response(
            None,
            &["1.1.1", "1.1.2", "1.2.2", "2.2.2"],
            "OSV-MINOR",
            "minor target issue",
            Some(("1.2.2", "1.2.3")),
        ),
    );
    let titles = lens_titles(&output);

    assert!(titles.contains(&"⚠️ minor 1.2.2"));
    assert!(!titles.contains(&"U minor 1.2.2"));
}

#[test]
fn vulnerable_build_code_lens_uses_vulnerable_update_indicator_fallback() {
    let output = vulnerable_output(
        session_with_empty_vulnerable_indicator(),
        "left-pad-1.0.0-b1.json",
        npm_vulnerability_response(
            Some("1.0.0+b2"),
            &["1.0.0+b1", "1.0.0+b2"],
            "OSV-BUILD",
            "build target issue",
            None,
        ),
    );
    let titles = lens_titles(&output);

    assert!(titles.contains(&"⚠️ change build"));
    assert!(!titles.contains(&"B change build"));
}

#[test]
fn update_choice_code_lens_marks_vulnerable_non_latest_targets() {
    let output = vulnerable_output(
        standard_session(),
        "left-pad-1.1.1.json",
        npm_vulnerability_response(
            None,
            &["1.1.1", "1.1.2", "1.2.2", "2.2.2"],
            "OSV-MINOR",
            "minor target issue",
            Some(("1.2.2", "1.2.3")),
        ),
    );
    let titles = lens_titles(&output);

    assert_eq!(
        titles,
        [
            "M fixed 1.1.1",
            "U patch 1.1.2",
            "V minor 1.2.2",
            "U latest 2.2.2"
        ]
    );
}
