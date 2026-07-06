use super::{DocumentInput, package_file_fixture, session_with_properties};
use versionlens_parsers::Ecosystem::Pub;

#[test]
fn dependency_properties_allow_custom_pub_paths() {
    let session = session_with_properties(Pub, &["custom_dependencies"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pubspec-custom-dependencies.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].name, "retry");
}

#[test]
fn pub_dependency_properties_allow_member_paths() {
    let session = session_with_properties(Pub, &["dependencies.http", "dev_dependencies.*"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pubspec-member-paths.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 2);
    assert_eq!(output.dependencies[0].name, "http");
    assert_eq!(output.dependencies[1].name, "test");
}

#[test]
fn pub_dependency_properties_allow_workspace_paths() {
    let session = session_with_properties(Pub, &["workspace"]);

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pubspec-workspace.yaml"),
        workspace_root: None,
    });

    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].group, "workspace");
    assert_eq!(output.dependencies[0].name, "packages/shared");
}
