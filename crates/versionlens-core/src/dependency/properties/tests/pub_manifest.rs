use super::{DocumentInput, Ecosystem, session_with_properties};

#[test]
fn dependency_properties_allow_custom_pub_paths() {
    let session = session_with_properties(Ecosystem::Pub, &["custom_dependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "\
dependencies:
  http: ^1.0.0
custom_dependencies:
  retry: ^1.0.0
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "retry");
}

#[test]
fn pub_dependency_properties_allow_member_paths() {
    let session =
        session_with_properties(Ecosystem::Pub, &["dependencies.http", "dev_dependencies.*"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "\
dependencies:
  http: ^1.0.0
  dio: ^5.0.0
dev_dependencies:
  test: ^1.25.0
"
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 2);
    assert_eq!(output.dependencies[0].name, "http");
    assert_eq!(output.dependencies[1].name, "test");
}
