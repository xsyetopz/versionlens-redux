pub(crate) fn matching_delimiter(text: &str, open: usize, left: u8, right: u8) -> Option<usize> {
    let mut depth = 0usize;
    let mut offset = open;
    let mut in_string = false;
    let mut escaped = false;
    while offset < text.len() {
        let byte = *text.as_bytes().get(offset)?;
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
        } else if byte == left {
            depth += 1;
        } else if byte == right {
            depth -= 1;
            if depth == 0 {
                return Some(offset);
            }
        }
        offset += 1;
    }
    None
}
