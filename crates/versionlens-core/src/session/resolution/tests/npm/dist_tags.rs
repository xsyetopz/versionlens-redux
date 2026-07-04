use super::{
    DocumentInput, Ecosystem, RegistryResponseInput, session_without_vulnerabilities,
    standard_session,
};

#[test]
fn npm_latest_dist_tag_caps_stable_update_choices() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"7.0.0"}}"#.to_owned(),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{
              "dist-tags": { "latest": "7.0.0" },
              "versions": {
                "7.0.0": {},
                "8.0.0": {}
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

    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("7.0.0"));
    assert!(output.edits.is_empty());
    assert_eq!(titles, ["🟢 latest 7.0.0"]);
}

#[test]
fn resolves_npm_dist_tag_requirements_against_dist_tags() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"typescript":"next"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "typescript".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"6.0.3","next":"7.0.0-beta.1"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#.to_owned(),
        }],
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
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"typescript":"missing-tag"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "typescript".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert!(output.edits.is_empty());
}
