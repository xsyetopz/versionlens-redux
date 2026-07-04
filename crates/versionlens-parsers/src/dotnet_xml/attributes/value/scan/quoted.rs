use std::ops::Range as ByteRange;

pub(super) fn is_attribute_quote(byte: u8) -> bool {
    byte == b'"' || byte == b'\''
}

pub(super) fn quoted_attribute_value_range(
    bytes: &[u8],
    mut index: usize,
    quote: u8,
) -> Option<(ByteRange<usize>, usize)> {
    let start = index;
    while index < bytes.len() && bytes[index] != quote {
        index += 1;
    }
    if index >= bytes.len() {
        return None;
    }
    Some((start..index, index + 1))
}
