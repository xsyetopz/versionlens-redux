use crate::positions::line_range;
use versionlens_model::Dependency;

use super::super::line::{GemLineContext, GemNameSpan, gem_name_range};
use super::super::syntax::attr_string_span;
use super::repository::github_repository;
use super::url::github_api_url;
use versionlens_model::Ecosystem::Ruby;

pub(in crate::gemfile) fn gem_github_tag_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
) -> Option<Dependency> {
    let (tag, attr_start, _, tag_end) = attr_string_span(context.content, "tag")?;
    let repo = github_repository(context.content)?;
    let quote = context
        .content
        .as_bytes()
        .get(tag_end)
        .copied()
        .unwrap_or(b'"') as char;

    Some(Dependency {
        name: repo.to_owned(),
        requirement: tag.into_owned(),
        ecosystem: Ruby,
        group: context.group.to_owned(),
        hosted_url: Some(github_api_url(&repo, "tags")),
        hosted_name: Some(name.name.to_owned()),
        range: gem_name_range(context),
        requirement_range: line_range(
            context.line_index,
            context.line,
            context.offset + attr_start,
            context.offset + tag_end + quote.len_utf8(),
        ),
        requirement_prefix: format!("tag: {quote}"),
        requirement_suffix: quote.to_string(),
    })
}
