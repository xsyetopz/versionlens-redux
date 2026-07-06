mod config;
mod input;
mod output;
mod position;

use versionlens_core::{
    AnalyzeDocumentOutput as CoreAnalyzeDocumentOutput,
    ResolveDocumentOutput as CoreResolveDocumentOutput,
};

pub(crate) use config::NativeSessionConfig;
pub(crate) use input::{NativeApplyCommandInput, NativeDocumentInput};
use output::NativeStatusPayload as StatusOutput;
pub(crate) use output::{NativeAnalyzeDocumentOutput, NativeResolveDocumentOutput};

pub(crate) fn empty_resolve_document_output() -> NativeResolveDocumentOutput {
    NativeResolveDocumentOutput {
        suggestions: vec![],
        edits: vec![],
        authorization_required_count: 0,
        authorization_required_requests: vec![],
        vulnerable_update_count: 0,
        vulnerable_update_package: None,
        vulnerable_update_version: None,
    }
}

pub(crate) fn resolve_document_output_from_core(
    output: CoreResolveDocumentOutput,
) -> NativeResolveDocumentOutput {
    output.into()
}

pub(crate) fn empty_analyze_document_output() -> NativeAnalyzeDocumentOutput {
    NativeAnalyzeDocumentOutput {
        can_sort_dependencies: false,
        is_supported_manifest: false,
        active_provider_name: None,
        install_task_config_key: None,
        dependency_signature: "".to_owned(),
        dependencies: vec![],
        code_lenses: vec![],
        diagnostics: vec![],
        status: StatusOutput {
            dependency_count: 0,
            update_count: 0,
            vulnerability_count: 0,
            error_count: 0,
            no_match_count: 0,
            visible: false,
            text: "".to_owned(),
            tooltip: "".to_owned(),
        },
    }
}

pub(crate) fn analyze_document_output_from_core(
    output: CoreAnalyzeDocumentOutput,
) -> NativeAnalyzeDocumentOutput {
    output.into()
}
