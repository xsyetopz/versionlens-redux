use versionlens_model::{Position, Range};

use super::to_u32;

pub(crate) fn offset_range(text: &str, start: usize, end: usize) -> Range {
    Range {
        start: offset_position(text, start),
        end: offset_position(text, end),
    }
}

fn offset_position(text: &str, offset: usize) -> Position {
    let prefix = prefix_at_byte_offset(text, offset);
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let character = prefix
        .rsplit_once('\n')
        .map(|(_, tail)| utf16_code_units(tail))
        .unwrap_or_else(|| utf16_code_units(prefix));

    Position {
        line: to_u32(line),
        character: to_u32(character),
    }
}

fn prefix_at_byte_offset(text: &str, offset: usize) -> &str {
    let mut end = offset.min(text.len());
    while !text.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    text.get(..end).unwrap_or("")
}

pub(super) fn utf16_code_units(value: &str) -> usize {
    value.chars().map(|value| value.len_utf16()).sum()
}
