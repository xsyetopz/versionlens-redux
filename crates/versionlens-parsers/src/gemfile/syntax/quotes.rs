pub(in crate::gemfile) fn quoted_strings(
    input: &str,
) -> impl Iterator<Item = (&str, usize, usize)> {
    let mut offset = 0;
    std::iter::from_fn(move || {
        let tail = &input[offset..];
        let quote_start = tail.find(['"', '\''])? + offset;
        let quote = input.as_bytes()[quote_start];
        let content_start = quote_start + 1;
        let content_tail = &input[content_start..];
        let content_end = content_tail
            .bytes()
            .position(|byte| byte == quote)
            .map(|index| content_start + index)?;
        offset = content_end + 1;
        Some((
            &input[content_start..content_end],
            content_start,
            content_end,
        ))
    })
}
