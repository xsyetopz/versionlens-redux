use std::borrow::Cow;

use super::quoted_strings;

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
                Cow::Borrowed(value),
                attr_start,
                start + value_start,
                start + value_end,
            )
        })
}

pub(in crate::gemfile) fn github_string(content: &str) -> Option<(Cow<'_, str>, usize, usize)> {
    let (repo, start, end) = attr_string(content, "github")?;
    Some((
        Cow::Owned(format!("https://github.com/{repo}.git")),
        start,
        end,
    ))
}
