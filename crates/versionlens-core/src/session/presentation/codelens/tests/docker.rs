use super::*;

#[test]
fn docker_argument_image_name_uses_not_supported_status() {
    let lenses = docker_code_lenses(package_file_fixture("Dockerfile-arg-image").as_str());

    assert_eq!(lenses, [("N not supported".to_owned(), "".to_owned())]);
}

#[test]
fn docker_argument_image_version_uses_not_supported_status() {
    let lenses = docker_code_lenses(package_file_fixture("Dockerfile-arg-version").as_str());

    assert_eq!(lenses, [("N not supported".to_owned(), "".to_owned())]);
}

const NODE_IMAGE_RESPONSE: &str = r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"current-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"current","tag_status":"active","digest":"sha256-23"},{"name":"bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"23.11-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11","tag_status":"active","digest":"sha256-23"},{"name":"23-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"},{"name":"22.4.3","tag_status":"active","digest":"sha256-22"},{"name":"22.4","tag_status":"active","digest":"sha256-22"},{"name":"22-bookworm","tag_status":"active","digest":"sha256-22"},{"name":"22","tag_status":"active","digest":"sha256-22"},{"name":"21.0.0","tag_status":"active","digest":"sha256-21"},{"name":"21.0","tag_status":"active","digest":"sha256-21"}]}"#;

fn docker_output(fixture: &str) -> crate::AnalyzeDocumentOutput {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///Dockerfile".to_owned(),
        "dockerfile".to_owned(),
        package_file_fixture(fixture),
        None,
    );
    crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new(
            "node".to_owned(),
            Docker,
            NODE_IMAGE_RESPONSE.to_owned(),
        )],
    )
}

fn docker_code_lenses(text: &str) -> Vec<(String, String)> {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///Dockerfile".to_owned(),
        "dockerfile".to_owned(),
        text.to_owned(),
        None,
    );

    session.resolve_document_with_responses(input.clone(), &[]);
    session
        .analyze_document(input)
        .code_lenses
        .into_iter()
        .map(|lens| (lens.title, lens.command))
        .collect()
}

#[test]
fn docker_code_lenses_offer_same_suffix_update_choices() {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///Dockerfile".to_owned(),
        "dockerfile".to_owned(),
        package_file_fixture("Dockerfile-node-20-bookworm"),
        None,
    );

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new("node".to_owned(), Docker, r#"{"results":[{"name":"20.19.1-bookworm","tag_status":"active","digest":"sha256-20-bookworm"},{"name":"21.0.0-alpine","tag_status":"active","digest":"sha256-21-alpine"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23-bookworm"}]}"#
                .to_owned())],
    );
    let arguments = crate::support::tests::update_code_lens_arguments(&output);

    assert_eq!(arguments, [vec!["update", "23.11.0-bookworm"]]);
}

#[test]
fn docker_code_lenses_map_latest_update_choice_to_matching_tag_shape() {
    let output = docker_output("Dockerfile-node-22-bookworm");
    let arguments = crate::support::tests::update_code_lens_arguments(&output);
    assert_eq!(arguments, [vec!["update", "23-bookworm"]]);
    assert_eq!(
        output
            .code_lenses
            .iter()
            .find(|lens| lens.command == "versionlens.suggestion.onChooseBuild")
            .map(|lens| lens
                .arguments
                .iter()
                .skip(1)
                .map(|value| value.as_str())
                .collect()),
        Some(vec![
            "node",
            "22-bookworm",
            "22",
            "22-bookworm",
            "22.4",
            "22.4.3"
        ])
    );
}

#[test]
fn docker_code_lenses_offer_update_choices_for_missing_numeric_tag() {
    let output = docker_output("Dockerfile-node-21");
    let titles = lens_titles(&output);
    let arguments = crate::support::tests::code_lens_arguments(&output);

    assert_eq!(titles, ["N no match", "U latest 23", "U major 22"]);
    assert_eq!(arguments, [vec!["update", "23"], vec!["updateMajor", "22"]]);
}

#[test]
fn docker_code_lenses_offer_latest_for_untagged_non_version_latest() {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///Dockerfile".to_owned(),
        "dockerfile".to_owned(),
        package_file_fixture("Dockerfile-mssql-server"),
        None,
    );

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new("mssql/server".to_owned(), Docker, r#"{"results":[{"name":"2022-RTM-CU2-ubuntu-20.04","tag_status":"active","digest":"sha256-a"},{"name":"2022-RTM-GDR1-ubuntu-20.04","tag_status":"active","digest":"sha256-b"},{"name":"2022-RTM-ubuntu-20.04","tag_status":"active","digest":"sha256-c"},{"name":"2022-latest","tag_status":"active","digest":"sha256-latest"},{"name":"2022-preview-ubuntu-22.04","tag_status":"active","digest":"sha256-d"},{"name":"latest","tag_status":"active","digest":"sha256-latest"},{"name":"latest-ubuntu","tag_status":"active","digest":"sha256-e"}]}"#
                .to_owned())],
    );
    let titles = lens_titles(&output);
    let arguments = crate::support::tests::code_lens_arguments(&output);

    assert_eq!(titles, ["N no match", "U latest latest"]);
    assert_eq!(arguments, [vec!["update", "latest"]]);
}
