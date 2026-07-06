use versionlens_parsers::DocumentInput;

use crate::AnalyzeDocumentOutput;
use crate::status::status_payload;

pub(crate) fn schema_output(input: &DocumentInput) -> AnalyzeDocumentOutput {
    let _ = input;
    let diagnostics = vec![];
    let status = status_payload(0, &diagnostics, &[], false);

    AnalyzeDocumentOutput {
        can_sort_dependencies: false,
        is_supported_manifest: true,
        active_provider_name: None,
        install_task_config_key: None,
        dependency_signature: "".to_owned(),
        dependencies: vec![],
        code_lenses: vec![],
        diagnostics,
        status,
    }
}

#[cfg(test)]
mod tests;
