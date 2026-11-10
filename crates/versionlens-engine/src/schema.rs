use versionlens_model::DocumentInput;

use crate::AnalyzeDocumentOutput;
use crate::presentation::status::status_view;

pub(crate) fn schema_output(input: &DocumentInput) -> AnalyzeDocumentOutput {
    let _ = input;
    let diagnostics = vec![];
    let status = status_view(0, &diagnostics, &[], false);

    AnalyzeDocumentOutput {
        can_sort_dependencies: false,
        is_supported_manifest: true,
        active_provider_name: None,
        install_task_config_key: None,
        dependency_signature: String::new(),
        dependencies: vec![],
        code_lenses: vec![],
        diagnostics,
        status,
    }
}

#[cfg(test)]
mod tests {
    use versionlens_model::DocumentInput;

    #[test]
    fn analyzes_extension_schema_documents_without_dependency_diagnostics() {
        let session = crate::test_support::test_session(false);
        let valid = session.analyze_document(DocumentInput::new("versionlens:/multi-registries.json".to_owned(), "json".to_owned(), package_file_fixture("analyzes-extension-schema-documents-without-dependency-diagnostics.multi-registries.json"), None));

        assert!(valid.is_supported_manifest);
        assert_eq!(valid.diagnostics.len(), 0);
        assert_eq!(valid.dependencies.len(), 0);
        assert!(!valid.can_sort_dependencies);

        let invalid = session.analyze_document(DocumentInput::new("versionlens:/multi-registries.json".to_owned(), "json".to_owned(), package_file_fixture(
            "analyzes-extension-schema-documents-without-dependency-diagnostics.multi-registries-2.txt",
        ), None));

        assert!(invalid.is_supported_manifest);
        assert_eq!(invalid.diagnostics.len(), 0);
    }

    fn package_file_fixture(name: &str) -> String {
        crate::test_support::fixture("tests/fixtures/core-scenarios/schema/tests", name)
    }
}
