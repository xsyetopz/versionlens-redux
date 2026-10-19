use versionlens_model::DocumentInput;

#[test]
fn analyzes_extension_schema_documents_without_dependency_diagnostics() {
    let session = crate::support::tests::test_session(false);
    let valid = session.analyze_document(DocumentInput::new("versionlens:/multi-registries.json".to_owned(), "json".to_owned(), package_file_fixture("analyzes-extension-schema-documents-without-dependency-diagnostics.multi-registries.json"), None));

    assert!(valid.is_supported_manifest);
    assert!(valid.diagnostics.is_empty());
    assert!(valid.dependencies.is_empty());
    assert!(!valid.can_sort_dependencies);

    let invalid = session.analyze_document(DocumentInput::new("versionlens:/multi-registries.json".to_owned(), "json".to_owned(), package_file_fixture(
            "analyzes-extension-schema-documents-without-dependency-diagnostics.multi-registries-2.txt",
        ), None));

    assert!(invalid.is_supported_manifest);
    assert!(invalid.diagnostics.is_empty());
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/core-scenarios/schema/tests", name)
}
