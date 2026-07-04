use versionlens_vscode_model::{Position, Range};

pub(crate) fn line_range(start_line: usize, end_line: usize, end_text: &str) -> Range {
    Range {
        start: Position {
            line: to_u32(start_line),
            character: 0,
        },
        end: Position {
            line: to_u32(end_line),
            character: to_u32(utf16_code_units(end_text)),
        },
    }
}

fn utf16_code_units(value: &str) -> usize {
    value.chars().map(char::len_utf16).sum()
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
