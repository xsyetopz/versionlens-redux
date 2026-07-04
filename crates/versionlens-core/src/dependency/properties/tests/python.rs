use super::{DocumentInput, Ecosystem, session_with_properties};

#[test]
fn dependency_properties_allow_custom_python_toml_paths() {
    let session = session_with_properties(Ecosystem::Python, &["tool.uv.sources"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: "\
[project]
dependencies = [\"requests>=2\"]

[tool.uv.sources]
custom = \"1.0.0\"
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "custom");
}
