pub(in crate::dotnet_xml) fn tag_bounds(
    text: &str,
    tag_start: usize,
    tag_end: usize,
) -> (usize, usize) {
    let offset = text
        .get(tag_start..tag_end)
        .and_then(|tag| tag.find('<'))
        .unwrap_or(0);
    (
        tag_start + offset,
        tag_end.saturating_add(offset).min(text.len()),
    )
}

pub(in crate::dotnet_xml) fn version_insert(tag: &str) -> Option<(usize, &'static str)> {
    let bytes = tag.as_bytes();
    let mut index = bytes.len().checked_sub(1)?;
    while index > 0 && bytes[index - 1].is_ascii_whitespace() {
        index -= 1;
    }
    if index > 0 && bytes[index - 1] == b'/' {
        index -= 1;
        while index > 0 && bytes[index - 1].is_ascii_whitespace() {
            index -= 1;
        }
    }
    let separator = if index > 0 && bytes[index - 1].is_ascii_whitespace() {
        ""
    } else {
        " "
    };
    Some((index, separator))
}
