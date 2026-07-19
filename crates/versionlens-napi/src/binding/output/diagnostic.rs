use napi_derive::napi;
use versionlens_vscode_model::DiagnosticPayload;

use crate::binding::position::{NativeRange, native_range_from_core};

#[napi(object)]
pub struct NativeDiagnosticPayload {
    pub range: NativeRange,
    pub message: String,
    pub severity: u8,
    pub source: Option<String>,
    pub code: Option<String>,
    pub code_description_url: Option<String>,
}

impl NativeDiagnosticPayload {
    pub(super) fn from_core(payload: DiagnosticPayload) -> Self {
        Self {
            range: native_range_from_core(payload.range),
            message: payload.message,
            severity: payload.severity,
            source: payload.source,
            code: payload.code,
            code_description_url: payload.code_description_url,
        }
    }
}

impl From<DiagnosticPayload> for NativeDiagnosticPayload {
    fn from(value: DiagnosticPayload) -> Self {
        Self::from_core(value)
    }
}
