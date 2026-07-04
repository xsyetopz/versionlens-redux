use versionlens_vscode_model::{Position, Range};

use super::offset::utf16_code_units;
use super::to_u32;

pub(crate) fn line_range(line_index: usize, line: &str, start: usize, end: usize) -> Range {
    Range {
        start: Position {
            line: to_u32(line_index),
            character: to_u32(line_character(line, start)),
        },
        end: Position {
            line: to_u32(line_index),
            character: to_u32(line_character(line, end)),
        },
    }
}

fn line_character(line: &str, offset: usize) -> usize {
    let mut end = offset.min(line.len());
    while !line.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    line.get(..end).map(utf16_code_units).unwrap_or(0)
}
