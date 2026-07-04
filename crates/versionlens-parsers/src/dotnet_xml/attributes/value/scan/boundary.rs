pub(super) fn skip_attribute_boundary(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len()
        && (bytes[index].is_ascii_whitespace() || matches!(bytes[index], b'/' | b'>'))
    {
        index += 1;
    }
    index
}

pub(super) fn skip_spaces(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    index
}
