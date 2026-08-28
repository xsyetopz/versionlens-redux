use super::*;

#[test]
fn code_lenses_label_latest_dist_tag_prerelease_for_missing_fixed_versions() {
    let session = standard_session();
    let input = package_document("left-pad-4.0.0.json");

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{
              "dist-tags": { "latest": "4.0.0-next" },
              "versions": {
                "0.0.5": {},
                "0.0.6": {},
                "1.1.0-alpha.1": {},
                "4.0.0-next": {}
              }
            }"#
            .to_owned(),
        )],
    );
    let titles = lens_titles(&output);
    let arguments = crate::support::tests::code_lens_arguments(&output);

    assert_eq!(titles, ["N no match", "U latest prerelease 4.0.0-next"]);
    assert_eq!(arguments, [vec!["update", "4.0.0-next"]]);
}

#[test]
fn npm_invalid_tag_name_error_offers_latest_dist_tag_update() {
    let session = standard_session();
    let input = package_document("left-pad-bad-tag.json");

    let resolved = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{"status":"EINVALIDTAGNAME"}"#.to_owned(),
        )],
    );
    let output = session.analyze_document(input);
    let titles = lens_titles(&output);
    let commands = lens_commands(&output);
    let arguments = crate::support::tests::code_lens_arguments(&output);

    assert_eq!(resolved.suggestions[0].status, "invalid");
    assert_eq!(titles, ["E invalid version", "U latest latest"]);
    assert_eq!(commands, ["", "versionlens.suggestion.onUpdateDependency"]);
    assert_eq!(arguments, [vec!["update", "latest"]]);
}

#[test]
fn npm_unsupported_protocol_error_uses_not_supported_status() {
    let session = standard_session();
    let input = package_document("left-pad-1.0.0.json");

    let resolved = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{"status":"EUNSUPPORTEDPROTOCOL"}"#.to_owned(),
        )],
    );
    let output = session.analyze_document(input);
    let titles = lens_titles(&output);
    let commands = lens_commands(&output);

    assert_eq!(resolved.suggestions[0].status, "notSupported");
    assert_eq!(titles, ["N not supported"]);
    assert_eq!(commands, [""]);
}
