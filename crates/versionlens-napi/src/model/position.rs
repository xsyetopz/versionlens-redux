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

fn native_position_from_core(position: Position) -> NativePosition {
    position.into()
}

pub(crate) fn native_range_from_core(range: Range) -> NativeRange {
    NativeRange {
        start: native_position_from_core(range.start),
        end: native_position_from_core(range.end),
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

impl From<Position> for NativePosition {
    fn from(value: Position) -> Self {
        Self::from_core(value)
    }
}
