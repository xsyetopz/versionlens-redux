use crate::positions::line_range;
use versionlens_model::Dependency;

use super::super::line::{GemLineContext, GemNameSpan, gem_name_range};
use super::super::syntax::attr_string_span;
use super::repository::github_repository;
use super::url::github_api_url;
use versionlens_model::Ecosystem::Ruby;

pub(in crate::gemfile) fn gem_github_ref_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
) -> Option<Dependency> {
    gem_github_value_dependency(context, name, "ref")
        .or_else(|| gem_github_value_dependency(context, name, "branch"))
}

fn gem_github_value_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
    attr_name: &str,
) -> Option<Dependency> {
    let (value, attr_start, value_start, value_end) = attr_string_span(context.content, attr_name)?;
    let repo = github_repository(context.content)?;

    let replacement = replacement_span(
        context.content,
        attr_name,
        attr_start,
        value_start,
        value_end,
    );

    Some(Dependency {
        name: repo.to_owned(),
        requirement: value.into_owned(),
        ecosystem: Ruby,
        group: context.group.to_owned(),
        hosted_url: Some(github_api_url(&repo, "commits")),
        hosted_name: Some(name.name.to_owned()),
        range: gem_name_range(context),
        requirement_range: line_range(
            context.line_index,
            context.line,
            context.offset + replacement.start,
            context.offset + replacement.end,
        ),
        requirement_prefix: replacement.prefix,
        requirement_suffix: replacement.suffix,
    })
}

struct ReplacementSpan {
    start: usize,
    end: usize,
    prefix: String,
    suffix: String,
}

fn replacement_span(
    content: &str,
    attr_name: &str,
    attr_start: usize,
    value_start: usize,
    value_end: usize,
) -> ReplacementSpan {
    if attr_name != "branch" {
        return ReplacementSpan {
            start: value_start,
            end: value_end,
            prefix: "".to_owned(),
            suffix: "".to_owned(),
        };
    }

    let quote = content.as_bytes().get(value_end).copied().unwrap_or(b'"') as char;
    ReplacementSpan {
        start: attr_start,
        end: value_end + quote.len_utf8(),
        prefix: format!("ref: {quote}"),
        suffix: quote.to_string(),
    }
}
