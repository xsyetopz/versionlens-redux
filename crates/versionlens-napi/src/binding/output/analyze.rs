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
            dependency_signature: "".to_owned(),
            dependencies: vec![],
            code_lenses: vec![],
            diagnostics: vec![],
            status: NativeStatusPayload {
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
                .map(|dependency| dependency.into())
                .collect(),
            code_lenses: output
                .code_lenses
                .into_iter()
                .map(|code_lens| code_lens.into())
                .collect(),
            diagnostics: output
                .diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.into())
                .collect(),
            status: output.status.into(),
        }
    }
}

impl Default for NativeAnalyzeDocumentOutput {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<AnalyzeDocumentOutput> for NativeAnalyzeDocumentOutput {
    fn from(value: AnalyzeDocumentOutput) -> Self {
        Self::from_core(value)
    }
}
