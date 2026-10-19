use super::*;

#[tokio::test]
async fn code_lenses_offer_minor_update_choices_for_tilde_ranges() {
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
    )
    .await;
    let orders = output
        .code_lenses
        .iter()
        .map(|lens| lens.order)
        .collect::<Vec<_>>();
    assert_eq!(orders, [0, 1, 2, 3]);
    assert!(
        output
            .code_lenses
            .iter()
            .all(|lens| lens.range == output.code_lenses[0].range),
        "lenses of one dependency share its range: {:?}",
        output.code_lenses
    );
    let titles = lens_titles(&output);
    let arguments = crate::support::tests::all_code_lens_arguments(&output);

    assert_eq!(
        titles,
        [
            "M satisfies 1.1.2",
            "U bump 1.1.2",
            "U minor 1.2.2",
            "U latest 2.2.2"
        ]
    );
    assert_eq!(
        arguments,
        [
            Vec::<&str>::new(),
            vec!["update", "1.1.2"],
            vec!["updateMinor", "1.2.2"],
            vec!["update", "2.2.2"]
        ]
    );
}
