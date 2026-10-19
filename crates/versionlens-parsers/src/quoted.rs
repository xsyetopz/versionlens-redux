#[derive(Clone, Copy)]
pub(crate) struct QuotedString<'a> {
    pub(crate) value: &'a str,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn double_quoted_string_at(text: &str, quote_start: usize) -> Option<QuotedString<'_>> {
    let value_start = quote_start + 1;
    let mut index = value_start;
    let bytes = text.as_bytes();
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Some(QuotedString {
                value: &text[value_start..index],
                start: value_start,
                end: index,
            });
        }
        index += 1;
    }
    None
}
