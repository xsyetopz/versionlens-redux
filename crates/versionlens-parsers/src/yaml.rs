use marked_yaml::types::MarkedScalarNode;
use std::ops::Range;

pub(crate) fn scalar_range(text: &str, value: &MarkedScalarNode) -> Option<Range<usize>> {
    let raw_start = byte_offset(text, value.span().start()?.character())?;
    if let Some(quote @ (b'"' | b'\'')) = text.as_bytes().get(raw_start) {
        return quoted_content_range(text, raw_start, *quote);
    }

    let raw_end = value
        .span()
        .end()
        .and_then(|marker| byte_offset(text, marker.character()))
        .unwrap_or(raw_start + value.as_str().len());
    Some(raw_start..raw_end)
}

fn quoted_content_range(text: &str, start: usize, quote: u8) -> Option<Range<usize>> {
    let bytes = text.as_bytes();
    let mut offset = start + 1;
    while let Some(byte) = bytes.get(offset) {
        if *byte == quote {
            if quote == b'\'' && bytes.get(offset + 1) == Some(&quote) {
                offset += 2;
                continue;
            }
            return Some(start + 1..offset);
        }
        offset += if quote == b'"' && *byte == b'\\' {
            2
        } else {
            1
        };
    }
    None
}

pub(crate) fn scalar_content_offset(
    text: &str,
    range: Range<usize>,
    decoded_offset: usize,
) -> Option<usize> {
    let quote = range
        .start
        .checked_sub(1)
        .and_then(|index| text.as_bytes().get(index))
        .copied();
    let content = text.get(range.clone())?;
    let mut raw = 0;
    let mut decoded = 0;
    while decoded < decoded_offset {
        let character = content.get(raw..)?.chars().next()?;
        let (raw_length, decoded_length) = if quote == Some(b'"') && character == '\\' {
            escape_lengths(content.get(raw + 1..)?)?
        } else if quote == Some(b'\'') && content.get(raw..)?.starts_with("''") {
            (2, 1)
        } else {
            if character == '\n' || character == '\r' {
                return None;
            }
            (character.len_utf8(), character.len_utf8())
        };
        raw += raw_length;
        decoded += decoded_length;
    }
    (decoded == decoded_offset && raw <= content.len()).then_some(range.start + raw)
}

fn escape_lengths(encoded: &str) -> Option<(usize, usize)> {
    let escape = encoded.chars().next()?;
    let digits = match escape {
        'x' => 2,
        'u' => 4,
        'U' => 8,
        _ => 0,
    };
    if digits != 0 {
        let code = u32::from_str_radix(encoded.get(1..1 + digits)?, 16).ok()?;
        return Some((2 + digits, char::from_u32(code)?.len_utf8()));
    }
    let length = match escape {
        '0' | 'a' | 'b' | 't' | '\t' | 'n' | 'v' | 'f' | 'r' | 'e' | ' ' | '"' | '/' | '\\' => 1,
        'N' | '_' => 2,
        'L' | 'P' => 3,
        _ => return None,
    };
    Some((2, length))
}

pub(crate) fn byte_offset(text: &str, character: usize) -> Option<usize> {
    if character == text.chars().count() {
        return Some(text.len());
    }
    text.char_indices().nth(character).map(|(index, _)| index)
}

#[cfg(test)]
mod tests;
