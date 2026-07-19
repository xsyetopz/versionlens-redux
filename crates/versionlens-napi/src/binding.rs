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

pub(crate) fn analyze_document_output_from_core(
    output: CoreAnalyzeDocumentOutput,
) -> NativeAnalyzeDocumentOutput {
    output.into()
}
