use super::{DocumentInput, session_without_vulnerabilities, standard_session};
use crate::RegistryResponseInput;
use versionlens_model::Ecosystem::Npm;

#[test]
fn npm_latest_dist_tag_caps_stable_update_choices() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("latest-dist-tag-caps-stable-update-choices.json"),
        None,
    );

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{
              "dist-tags": { "latest": "7.0.0" },
              "versions": {
                "7.0.0": {},
                "8.0.0": {}
              }
            }"#
            .to_owned(),
        )],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "current", Some("7.0.0"));
    assert_eq!(titles, ["🟢 latest 7.0.0"]);
}

#[test]
fn resolves_npm_dist_tag_requirements_against_dist_tags() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("resolves-npm-dist-tag-requirements-against-dist-tags.json"), None),
        &[RegistryResponseInput::new("typescript".to_owned(), Npm, r#"{"dist-tags":{"latest":"6.0.3","next":"7.0.0-beta.1"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#.to_owned())],
    );

    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("7.0.0-beta.1")
    );
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "7.0.0-beta.1");
}

#[test]
fn missing_npm_dist_tag_requirement_resolves_no_match() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("missing-npm-dist-tag-requirement-resolves-no-match.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "typescript".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{}}}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture(
        "tests/fixtures/session/resolution/tests/npm/dist_tags",
        name,
    )
}
