use super::{DocumentInput, RegistryResponseInput, standard_session};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::Ecosystem::Dotnet;

#[test]
fn dotnet_nuget_versions_return_registry_suggestions() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/app.csproj".to_owned(),
            language_id: "xml".to_owned(),
            text: package_file_fixture("dotnet-nuget-versions-return-registry-suggestions.csproj"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "Microsoft.Extensions.Logging".to_owned(),
            ecosystem: Dotnet,
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
            text: package_file_fixture(
                "dotnet-four-segment-versions-return-empty-suggestions-from-registry.csproj",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "Test.Package".to_owned(),
            ecosystem: Dotnet,
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
        text: package_file_fixture("dotnet-invalid-versions-return-no-match-from-registry.csproj"),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "Test.Package".to_owned(),
            ecosystem: Dotnet,
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
                .map(|value| value.as_str())
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

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/fixed/dotnet")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
