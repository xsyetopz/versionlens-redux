use super::*;

pub(super) async fn resolve_github_fixture(
    fixture: &str,
    package: &str,
    ecosystem: Ecosystem,
    body: &str,
) -> ResolveDocumentOutput {
    let session = standard_session();
    let (uri, language) = if fixture.ends_with("Gemfile") {
        ("file:///Gemfile", "ruby")
    } else {
        ("file:///package.json", "json")
    };
    session
        .resolve_document_with_responses(
            DocumentInput::new(
                uri.to_owned(),
                language.to_owned(),
                package_file_fixture(fixture),
                None,
            ),
            &[RegistryResponseInput::new(
                package.to_owned(),
                ecosystem,
                body.to_owned(),
            )],
        )
        .await
}

pub(super) fn assert_no_action_update(output: &ResolveDocumentOutput) {
    assert!(output.edits.is_empty());
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status != "updateAvailable")
    );
}
