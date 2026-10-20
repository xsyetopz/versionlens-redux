use std::borrow::Cow;
use std::borrow::Cow::{Borrowed as CowBorrowed, Owned as CowOwned};
use std::iter::from_fn;

pub(in crate::gemfile) fn attr_string<'a>(
    content: &'a str,
    name: &str,
) -> Option<(Cow<'a, str>, usize, usize)> {
    let (value, _, value_start, value_end) = attr_string_span(content, name)?;
    Some((value, value_start, value_end))
}

pub(in crate::gemfile) fn attr_string_span<'a>(
    content: &'a str,
    name: &str,
) -> Option<(Cow<'a, str>, usize, usize, usize)> {
    let attr = format!("{name}:");
    let attr_start = content.find(&attr)?;
    let start = attr_start + attr.len();
    quoted_strings(&content[start..])
        .next()
        .map(|(value, value_start, value_end)| {
            (
                CowBorrowed(value),
                attr_start,
                start + value_start,
                start + value_end,
            )
        })
}

pub(in crate::gemfile) fn github_string(content: &str) -> Option<(Cow<'_, str>, usize, usize)> {
    let (repo, start, end) = attr_string(content, "github")?;
    Some((
        CowOwned(format!("https://github.com/{repo}.git")),
        start,
        end,
    ))
}

pub(in crate::gemfile) fn strip_comment(input: &str) -> &str {
    let mut quote = None;

    for (index, byte) in input.bytes().enumerate() {
        if byte == b'#' && quote.is_none() {
            return &input[..index];
        }

        update_comment_quote(byte, &mut quote);
    }

    input
}

fn update_comment_quote(byte: u8, quote: &mut Option<u8>) {
    match (*quote, byte) {
        (None, b'\'' | b'"') => *quote = Some(byte),
        (Some(open), current) if current == open => *quote = None,
        _ => {}
    }
}

pub(in crate::gemfile) fn quoted_strings(
    input: &str,
) -> impl Iterator<Item = (&str, usize, usize)> {
    let mut offset = 0;
    from_fn(move || {
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
