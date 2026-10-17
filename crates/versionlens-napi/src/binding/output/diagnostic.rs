use napi_derive::napi;
use versionlens_core::{DiagnosticView, Severity};

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
    pub(super) fn from_core(payload: DiagnosticView) -> Self {
        Self {
            range: native_range_from_core(payload.range),
            message: payload.message,
            severity: native_severity(payload.severity),
            source: payload.source,
            code: payload.code,
            code_description_url: payload.code_description_url,
        }
    }
}

impl From<DiagnosticView> for NativeDiagnosticPayload {
    fn from(value: DiagnosticView) -> Self {
        Self::from_core(value)
    }
}

/// VS Code `DiagnosticSeverity` numbering, which the extension passes through.
const fn native_severity(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Information => 2,
        Severity::Hint => 3,
    }
}
