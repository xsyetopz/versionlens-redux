use super::*;

#[test]
fn code_lenses_offer_minor_update_choices_for_tilde_ranges() {
    let session = standard_session();
    let input = package_document("left-pad-tilde-1.1.json");

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{
              "dist-tags": { "latest": "2.2.2" },
              "versions": {
                "1.1.0": {},
                "1.1.1": {},
                "1.1.2": {},
                "1.2.0": {},
                "1.2.2": {},
                "2.0.0": {},
                "2.2.2": {}
              }
            }"#
            .to_owned(),
        )],
    );
    let titles = lens_titles(&output);
    let arguments = crate::support::tests::all_code_lens_arguments(&output);

    assert_eq!(
        titles,
        [
            "M satisfies 1.1.2",
            "U bump 1.1.2",
            "U version 1.2.0",
            "U minor 1.2.2",
            "U version 2.0.0",
            "U latest 2.2.2"
        ]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "1.1.2"],
            vec!["update", "1.2.0"],
            vec!["updateMinor", "1.2.2"],
            vec!["update", "2.0.0"],
            vec!["update", "2.2.2"]
        ]
    );
}
