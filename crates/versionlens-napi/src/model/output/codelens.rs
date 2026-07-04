use napi_derive::napi;
use versionlens_vscode_model::CodeLensPayload;

use crate::model::position::NativeRange;

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
            range: NativeRange::from_core(payload.range),
            title: payload.title,
            command: payload.command,
            arguments: payload.arguments,
        }
    }
}
