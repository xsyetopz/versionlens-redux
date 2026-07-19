use napi_derive::napi;
use versionlens_model::TextEdit;

use crate::binding::position::{NativeRange, native_range_from_core};

#[napi(object)]
pub struct NativeTextEdit {
    pub range: NativeRange,
    pub new_text: String,
}

impl NativeTextEdit {
    pub(super) fn from_core(edit: TextEdit) -> Self {
        Self {
            range: native_range_from_core(edit.range),
            new_text: edit.new_text,
        }
    }
}

impl From<TextEdit> for NativeTextEdit {
    fn from(value: TextEdit) -> Self {
        Self::from_core(value)
    }
}
