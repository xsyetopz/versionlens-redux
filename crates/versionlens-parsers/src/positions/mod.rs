use versionlens_model::{Position, Range};

pub(crate) fn empty_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}

pub(crate) fn dependency_ranges(
    text: &str,
    name: Option<std::ops::Range<usize>>,
    requirement: Option<std::ops::Range<usize>>,
    requirement_is_quoted: bool,
) -> (Range, Range) {
    let name_range = name
        .map(|span| offset_range(text, span.start, span.end))
        .unwrap_or_else(empty_range);
    let requirement_range = requirement
        .map(|span| {
            let span = if requirement_is_quoted {
                string_content_bounds(text, span.start, span.end)
            } else {
                span
            };
            offset_range(text, span.start, span.end)
        })
        .unwrap_or(name_range);
    (name_range, requirement_range)
}

#[cfg(test)]
mod tests;

pub(crate) fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

pub(crate) fn to_usize(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

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

pub(crate) fn string_content_bounds(
    text: &str,
    start: usize,
    end: usize,
) -> std::ops::Range<usize> {
    let content_start = start + usize::from(text.as_bytes().get(start) == Some(&b'"'));
    let content_end = end.saturating_sub(usize::from(
        end > start && text.as_bytes().get(end - 1) == Some(&b'"'),
    ));
    content_start..content_end
}

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
