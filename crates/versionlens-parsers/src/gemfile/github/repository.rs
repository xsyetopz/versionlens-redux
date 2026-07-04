use std::borrow::Cow;

use super::super::syntax::attr_string_span;

pub(super) fn github_repository(content: &str) -> Option<&str> {
    borrowed_attr_string(content, "github")
        .and_then(normalize_github_repository)
        .or_else(|| borrowed_attr_string(content, "git").and_then(normalize_github_repository))
}

pub(super) fn normalize_github_repository(value: &str) -> Option<&str> {
    let value = value
        .split_once('#')
        .map_or(value, |(before_fragment, _)| before_fragment);
    let value = value.strip_suffix(".git").unwrap_or(value);
    let value = strip_github_prefix(value).unwrap_or(value);
    let value = value.strip_suffix('/').unwrap_or(value);
    let (owner, repo) = value.split_once('/')?;
    (!owner.is_empty() && !repo.is_empty() && !repo.contains('/')).then_some(value)
}

fn borrowed_attr_string<'a>(content: &'a str, name: &str) -> Option<&'a str> {
    let (value, _, _, _) = attr_string_span(content, name)?;
    match value {
        Cow::Borrowed(value) => Some(value),
        Cow::Owned(_) => None,
    }
}

fn strip_github_prefix(value: &str) -> Option<&str> {
    const PREFIXES: &[&str] = &[
        "github:",
        "https://github.com/",
        "http://github.com/",
        "git://github.com/",
        "git+https://github.com/",
        "git+ssh://git@github.com/",
        "ssh://git@github.com/",
        "git@github.com:",
    ];

    PREFIXES
        .iter()
        .find_map(|prefix| value.strip_prefix(prefix))
}
