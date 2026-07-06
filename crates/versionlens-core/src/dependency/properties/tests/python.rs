use super::{DocumentInput, package_file_fixture, session_with_properties};
use versionlens_parsers::Ecosystem::Python;

#[test]
fn dependency_properties_allow_custom_python_toml_paths() {
    let session = session_with_properties(Python, &["tool.uv.sources"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("pyproject-uv-sources.toml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "custom");
}
