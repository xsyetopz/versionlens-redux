use versionlens_parsers::DocumentInput;

use crate::AnalyzeDocumentOutput;
use crate::status::status_payload;

pub(crate) fn schema_output(input: &DocumentInput) -> AnalyzeDocumentOutput {
    let _ = input;
    let diagnostics = Vec::new();
    let status = status_payload(0, &diagnostics, &[], false);

    AnalyzeDocumentOutput {
        can_sort_dependencies: false,
        is_supported_manifest: true,
        active_provider_name: None,
        install_task_config_key: None,
        dependency_signature: String::new(),
        dependencies: Vec::new(),
        code_lenses: Vec::new(),
        diagnostics,
        status,
    }
}

#[cfg(test)]
mod tests;
