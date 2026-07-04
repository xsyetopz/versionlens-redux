use marked_yaml::types::MarkedScalarNode;

pub(crate) fn scalar_range(text: &str, value: &MarkedScalarNode) -> Option<std::ops::Range<usize>> {
    let raw_start = byte_offset(text, value.span().start()?.character())?;
    if matches!(text.as_bytes().get(raw_start), Some(b'"' | b'\'')) {
        let start = raw_start + 1;
        return Some(start..start + value.as_str().len());
    }

    let raw_end = value
        .span()
        .end()
        .and_then(|marker| byte_offset(text, marker.character()))
        .unwrap_or(raw_start + value.as_str().len());
    Some(raw_start..raw_end)
}

pub(crate) fn byte_offset(text: &str, character: usize) -> Option<usize> {
    if character == text.chars().count() {
        return Some(text.len());
    }
    text.char_indices().nth(character).map(|(index, _)| index)
}
