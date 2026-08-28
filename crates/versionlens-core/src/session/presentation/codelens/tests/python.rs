use super::*;

fn assert_python_code_lenses(output: &crate::AnalyzeDocumentOutput, expected: &[(&str, &str)]) {
    assert_eq!(
        output
            .code_lenses
            .iter()
            .map(|lens| (lens.title.as_str(), lens.command.as_str()))
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn pyproject_ranges_that_admit_latest_offer_updates_from_canonical_pypi_releases() {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///pyproject.toml".to_owned(),
        "toml".to_owned(),
        r#"
[project]
dependencies = ["httpx>=0.27,<1"]

[project.optional-dependencies]
test = ["httpcore>=0.27,<1"]
"#
        .to_owned(),
        None,
    );
    let body = r#"{
      "info": { "version": "0.28.1" },
      "releases": {
        "0.27.0": [],
        "0.28.1": [{ "yanked": false }]
      }
    }"#
    .to_owned();

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[
            RegistryResponseInput::new("httpx".to_owned(), Python, body.clone()),
            RegistryResponseInput {
                package: "httpcore".to_owned(),
                ecosystem: Python,
                body,
            },
        ],
    );

    assert_eq!(
        output
            .suggestions
            .iter()
            .map(|suggestion| suggestion.status.as_str())
            .collect::<Vec<_>>(),
        ["satisfiesLatest", "satisfiesLatest"]
    );

    let lenses = session.analyze_document(input);
    assert_python_code_lenses(
        &lenses,
        &[
            ("S satisfies latest 0.28.1", ""),
            (
                "U latest 0.28.1",
                "versionlens.suggestion.onUpdateDependency",
            ),
            ("S satisfies latest 0.28.1", ""),
            (
                "U latest 0.28.1",
                "versionlens.suggestion.onUpdateDependency",
            ),
        ],
    );
}

#[test]
fn requirements_ranges_offer_updates_from_canonical_pypi_releases() {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///requirements.txt".to_owned(),
        "pip-requirements".to_owned(),
        "httpx>=0.27,<1\n".to_owned(),
        None,
    );

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "httpx".to_owned(),
            Python,
            r#"{"info":{"version":"0.28.1"},"releases":{"0.27.0":[],"0.28.1":[{"yanked":false}]}}"#
                .to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].status, "satisfiesLatest");
    let lenses = session.analyze_document(input);
    assert_python_code_lenses(
        &lenses,
        &[
            ("S satisfies latest 0.28.1", ""),
            (
                "U latest 0.28.1",
                "versionlens.suggestion.onUpdateDependency",
            ),
        ],
    );
}
