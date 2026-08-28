use super::{DocumentInput, standard_session};
use crate::RegistryResponseInput;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn dotnet_nuget_versions_return_registry_suggestions() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/app.csproj".to_owned(),
            "xml".to_owned(),
            package_file_fixture("dotnet-nuget-versions-return-registry-suggestions.csproj"),
            None,
        ),
        &[RegistryResponseInput::new(
            "Microsoft.Extensions.Logging".to_owned(),
            Dotnet,
            r#"{"versions":["5.0.0","5.0.1"]}"#.to_owned(),
        )],
    );

    assert_eq!(
        output.suggestions[0].dependency.name,
        "Microsoft.Extensions.Logging"
    );
    assert_eq!(output.suggestions[0].dependency.requirement, "5.0.0");
    crate::support::tests::assert_suggestion(&output, 0, "fixed", Some("5.0.0"));
}

#[test]
fn dotnet_four_segment_versions_return_empty_suggestions_from_registry() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/app.csproj".to_owned(),
            "xml".to_owned(),
            package_file_fixture(
                "dotnet-four-segment-versions-return-empty-suggestions-from-registry.csproj",
            ),
            None,
        ),
        &[RegistryResponseInput::new(
            "Test.Package".to_owned(),
            Dotnet,
            r#"{"versions":["1.2.4"]}"#.to_owned(),
        )],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn dotnet_invalid_versions_return_no_match_from_registry() {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///repo/app.csproj".to_owned(),
        "xml".to_owned(),
        package_file_fixture("dotnet-invalid-versions-return-no-match-from-registry.csproj"),
        None,
    );

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "Test.Package".to_owned(),
            Dotnet,
            r#"{"versions":["1.2.4","1.3.0","2.0.0","2.1.0-beta.1"]}"#.to_owned(),
        )],
    );
    let analysis = session.analyze_document(input);
    let (titles, arguments) =
        crate::session::resolution::tests::lens_titles_and_arguments(&analysis);

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert_eq!(output.suggestions[0].dependency.name, "Test.Package");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());
    assert_eq!(titles, ["⚪ no match", "↑  latest 2.0.0"]);
    assert_eq!(arguments, [vec!["update", "2.0.0"]]);
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/fixed/dotnet", name)
}
