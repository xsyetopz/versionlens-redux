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
    let name_range = name.map_or_else(empty_range, |span| offset_range(text, span.start, span.end));
    let requirement_range = requirement.map_or(name_range, |span| {
        let span = if requirement_is_quoted {
            string_content_bounds(text, span.start, span.end)
        } else {
            span
        };
        offset_range(text, span.start, span.end)
    });
    (name_range, requirement_range)
}

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
    let character = prefix.rsplit_once('\n').map_or_else(
        || utf16_code_units(prefix),
        |(_, tail)| utf16_code_units(tail),
    );

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
    value.chars().map(char::len_utf16).sum()
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
    line.get(..end).map_or(0, utf16_code_units)
}

#[cfg(test)]
mod tests {
    use super::{line_range, offset_range};

    #[test]
    fn offset_ranges_count_utf16_code_units_like_vscode_position_at() {
        let text = package_file_fixture(
            "offset-ranges-count-utf16-code-units-like-vscode-position-at.txt",
        );
        let name_start = text.find("left-pad").expect("dependency name");
        let range = offset_range(text, name_start, name_start + "left-pad".len());

        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 20);
        assert_eq!(range.end.character, 28);
    }

    #[test]
    fn line_ranges_count_utf16_code_units_like_vscode_position_at() {
        let line = "😀 FROM node:20";
        let name_start = line.find("node").expect("image name");
        let range = line_range(0, line, name_start, name_start + "node".len());

        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 8);
        assert_eq!(range.end.character, 12);
    }

    fn package_file_fixture(name: &str) -> &'static str {
        crate::support::tests::fixture(
            "tests/fixtures/versionlens-parsers/src/positions/tests",
            name,
        )
    }
}
