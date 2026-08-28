use super::{DocumentInput, package_file_fixture, session_with_properties};
use versionlens_model::Ecosystem::Python;

#[test]
fn dependency_properties_allow_custom_python_toml_paths() {
    let session = session_with_properties(Python, &["tool.uv.sources"]);

    let output = session.analyze_document(DocumentInput::new(
        "file:///pyproject.toml".to_owned(),
        "toml".to_owned(),
        package_file_fixture("pyproject-uv-sources.toml"),
        None,
    ));

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "custom");
}
