use napi_derive::napi;
use versionlens_core::AnalyzeDocumentOutput;

use super::codelens::NativeCodeLensPayload;
use super::dependency::NativeDependency;
use super::diagnostic::NativeDiagnosticPayload;
use super::status::NativeStatusPayload;

#[napi(object)]
pub struct NativeAnalyzeDocumentOutput {
    pub can_sort_dependencies: bool,
    pub is_supported_manifest: bool,
    pub active_provider_name: Option<String>,
    pub install_task_config_key: Option<String>,
    pub dependency_signature: String,
    pub dependencies: Vec<NativeDependency>,
    pub code_lenses: Vec<NativeCodeLensPayload>,
    pub diagnostics: Vec<NativeDiagnosticPayload>,
    pub status: NativeStatusPayload,
}

impl NativeAnalyzeDocumentOutput {
    pub(crate) fn empty() -> Self {
        Self {
            can_sort_dependencies: false,
            is_supported_manifest: false,
            active_provider_name: None,
            install_task_config_key: None,
            dependency_signature: String::new(),
            dependencies: Vec::new(),
            code_lenses: Vec::new(),
            diagnostics: Vec::new(),
            status: NativeStatusPayload::empty(),
        }
    }
    pub(crate) fn from_core(output: AnalyzeDocumentOutput) -> Self {
        Self {
            can_sort_dependencies: output.can_sort_dependencies,
            is_supported_manifest: output.is_supported_manifest,
            active_provider_name: output.active_provider_name,
            install_task_config_key: output.install_task_config_key,
            dependency_signature: output.dependency_signature,
            dependencies: output
                .dependencies
                .into_iter()
                .map(NativeDependency::from_core)
                .collect(),
            code_lenses: output
                .code_lenses
                .into_iter()
                .map(NativeCodeLensPayload::from_core)
                .collect(),
            diagnostics: output
                .diagnostics
                .into_iter()
                .map(NativeDiagnosticPayload::from_core)
                .collect(),
            status: NativeStatusPayload::from_core(output.status),
        }
    }
}
