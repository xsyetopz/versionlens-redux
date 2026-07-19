use napi_derive::napi;
use versionlens_vscode_model::CodeLensPayload;

use crate::binding::position::{NativeRange, native_range_from_core};

#[napi(object)]
pub struct NativeCodeLensPayload {
    pub range: NativeRange,
    pub title: String,
    pub command: String,
    pub arguments: Vec<String>,
}

impl NativeCodeLensPayload {
    pub(super) fn from_core(payload: CodeLensPayload) -> Self {
        Self {
            range: native_range_from_core(payload.range),
            title: payload.title,
            command: payload.command,
            arguments: payload.arguments,
        }
    }
}

impl From<CodeLensPayload> for NativeCodeLensPayload {
    fn from(value: CodeLensPayload) -> Self {
        Self::from_core(value)
    }
}
