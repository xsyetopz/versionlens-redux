use std::ops::Range as ByteRange;

pub(super) fn attribute_name_range(bytes: &[u8], mut index: usize) -> (ByteRange<usize>, usize) {
    let start = index;
    while index < bytes.len()
        && !bytes[index].is_ascii_whitespace()
        && !matches!(bytes[index], b'=' | b'/' | b'>')
    {
        index += 1;
    }
    (start..index, index)
}
