use napi_derive::napi;
use versionlens_vscode_model::{Position, Range};

#[napi(object)]
pub struct NativeRange {
    pub start: NativePosition,
    pub end: NativePosition,
}

#[napi(object)]
pub struct NativePosition {
    pub line: u32,
    pub character: u32,
}

impl NativeRange {
    pub(crate) fn from_core(range: Range) -> Self {
        Self {
            start: NativePosition::from_core(range.start),
            end: NativePosition::from_core(range.end),
        }
    }
}

impl NativePosition {
    fn from_core(position: Position) -> Self {
        Self {
            line: position.line,
            character: position.character,
        }
    }
}
