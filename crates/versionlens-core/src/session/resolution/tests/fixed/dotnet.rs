use super::{DocumentInput, Ecosystem, RegistryResponseInput, standard_session};

#[test]
fn dotnet_nuget_versions_return_registry_suggestions() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/app.csproj".to_owned(),
            language_id: "xml".to_owned(),
            text: r#"<Project><ItemGroup><PackageReference Include="Microsoft.Extensions.Logging" Version="5.0.0" /></ItemGroup></Project>"#
                .to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "Microsoft.Extensions.Logging".to_owned(),
            ecosystem: Ecosystem::Dotnet,
            body: r#"{"versions":["5.0.0","5.0.1"]}"#.to_owned(),
        }],
    );

    assert_eq!(
        output.suggestions[0].dependency.name,
        "Microsoft.Extensions.Logging"
    );
    assert_eq!(output.suggestions[0].dependency.requirement, "5.0.0");
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("5.0.0"));
}

#[test]
fn dotnet_four_segment_versions_return_empty_suggestions_from_registry() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/app.csproj".to_owned(),
            language_id: "xml".to_owned(),
            text: r#"<Project><ItemGroup><PackageReference Include="Test.Package" Version="1.2.3.4" /></ItemGroup></Project>"#
                .to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "Test.Package".to_owned(),
            ecosystem: Ecosystem::Dotnet,
            body: r#"{"versions":["1.2.4"]}"#.to_owned(),
        }],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn dotnet_invalid_versions_return_no_match_from_registry() {
    let session = standard_session();
    let input = DocumentInput {
        uri: "file:///repo/app.csproj".to_owned(),
        language_id: "xml".to_owned(),
        text: r#"<Project><ItemGroup><PackageReference Include="Test.Package" Version="invalid" /></ItemGroup></Project>"#
            .to_owned(),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "Test.Package".to_owned(),
            ecosystem: Ecosystem::Dotnet,
            body: r#"{"versions":["1.2.4","1.3.0","2.0.0","2.1.0-beta.1"]}"#.to_owned(),
        }],
    );
    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert_eq!(output.suggestions[0].dependency.name, "Test.Package");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());
    assert_eq!(titles, ["⚪ no match", "↑  latest 2.0.0"]);
    assert_eq!(arguments, [vec!["update", "2.0.0"]]);
}
