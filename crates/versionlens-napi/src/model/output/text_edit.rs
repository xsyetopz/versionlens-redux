use napi_derive::napi;
use versionlens_vscode_model::TextEdit;

use crate::model::position::NativeRange;

#[napi(object)]
pub struct NativeTextEdit {
    pub range: NativeRange,
    pub new_text: String,
}

impl NativeTextEdit {
    pub(super) fn from_core(edit: TextEdit) -> Self {
        Self {
            range: NativeRange::from_core(edit.range),
            new_text: edit.new_text,
        }
    }
}
