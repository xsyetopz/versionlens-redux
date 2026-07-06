pub(super) fn coerced_version(raw: &str) -> Option<&str> {
    let start = raw.find(|char: char| char.is_ascii_digit())?;
    raw.get(start..start + coerced_version_len(&raw[start..])?)
}

fn coerced_version_len(raw: &str) -> Option<usize> {
    let bytes = raw.as_bytes();
    let mut end = read_digits(bytes, 0)?;
    for _ in 0..2 {
        if bytes.get(end) != Some(&b'.') {
            break;
        }
        end = read_digits(bytes, end + 1)?;
    }
    if bytes.get(end) == Some(&b'-') {
        end += 1;
        while bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
        {
            end += 1;
        }
    }
    Some(end)
}

fn read_digits(bytes: &[u8], start: usize) -> Option<usize> {
    let mut end = start;
    while bytes.get(end).is_some_and(|value| value.is_ascii_digit()) {
        end += 1;
    }
    (end > start).then_some(end)
}
