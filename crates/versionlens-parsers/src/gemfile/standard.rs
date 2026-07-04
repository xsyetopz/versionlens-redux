use std::borrow::Cow;

use crate::{
    model::{Dependency, Ecosystem},
    positions::line_range,
};

use super::line::{GemLineContext, GemNameSpan, gem_name_range};
use super::syntax::{attr_string, github_string};

pub(super) fn standard_gem_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
    version: Option<(&str, usize, usize)>,
    source_url: Option<&str>,
) -> Dependency {
    let (requirement, requirement_start, requirement_end, requirement_prefix, requirement_suffix) =
        gem_requirement(context.content, name.end, version);
    let hosted_url = gem_source_url(context.content).or_else(|| source_url.map(str::to_owned));

    Dependency {
        name: name.name.to_owned(),
        requirement: requirement.into_owned(),
        ecosystem: Ecosystem::Ruby,
        group: context.group.to_owned(),
        hosted_url,
        hosted_name: None,
        range: gem_name_range(context, name),
        requirement_range: line_range(
            context.line_index,
            context.line,
            context.offset + requirement_start,
            context.offset + requirement_end,
        ),
        requirement_prefix,
        requirement_suffix,
    }
}

fn gem_source_url(content: &str) -> Option<String> {
    attr_string(content, "source")
        .map(|(url, _, _)| url)
        .map(|url| url.trim_end_matches('/').to_owned())
        .filter(|url| !url.is_empty())
}

fn gem_requirement<'a>(
    content: &'a str,
    name_end: usize,
    version: Option<(&'a str, usize, usize)>,
) -> (Cow<'a, str>, usize, usize, String, String) {
    if let Some((requirement, start, end)) = version
        && !content[name_end..start].contains(':')
    {
        return (
            Cow::Borrowed(requirement),
            start,
            end,
            String::new(),
            String::new(),
        );
    }

    if let Some((requirement, start, end)) = attr_string(content, "path")
        .or_else(|| attr_string(content, "git"))
        .or_else(|| github_string(content))
    {
        return (requirement, start, end, String::new(), String::new());
    }

    let quote = content.as_bytes().get(name_end).copied().unwrap_or(b'"') as char;
    let insert_at = name_end + quote.len_utf8();
    (
        Cow::Borrowed("*"),
        insert_at,
        insert_at,
        format!(", {quote}"),
        quote.to_string(),
    )
}
