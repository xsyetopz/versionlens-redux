use crate::{
    model::{Dependency, Ecosystem},
    positions::line_range,
};

use super::super::line::{GemLineContext, GemNameSpan, gem_name_range};
use super::super::syntax::attr_string_span;
use super::repository::normalize_github_repository;
use super::url::github_api_url;

pub(in crate::gemfile) fn gem_github_default_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
) -> Option<Dependency> {
    if has_explicit_github_requirement(context.content) {
        return None;
    }

    default_github_ref_dependency(context, name)
        .or_else(|| default_git_ref_dependency(context, name))
}

fn default_github_ref_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
) -> Option<Dependency> {
    let (repo, _, _, repo_end) = attr_string_span(context.content, "github")?;
    let repo = normalize_github_repository(repo.as_ref())?;
    Some(default_dependency(
        context, name, repo, repo_end, "commits", "ref",
    ))
}

fn default_git_ref_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
) -> Option<Dependency> {
    let (repo, _, _, repo_end) = attr_string_span(context.content, "git")?;
    let repo = normalize_github_repository(repo.as_ref())?;
    Some(default_dependency(
        context, name, repo, repo_end, "commits", "ref",
    ))
}

fn default_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
    repo: &str,
    repo_end: usize,
    github_path: &str,
    inserted_attr: &str,
) -> Dependency {
    let quote = context
        .content
        .as_bytes()
        .get(repo_end)
        .copied()
        .unwrap_or(b'"') as char;
    let insert_at = context.content.len();
    Dependency {
        name: repo.to_owned(),
        requirement: String::new(),
        ecosystem: Ecosystem::Ruby,
        group: context.group.to_owned(),
        hosted_url: Some(github_api_url(repo, github_path)),
        hosted_name: Some(name.name.to_owned()),
        range: gem_name_range(context, name),
        requirement_range: line_range(
            context.line_index,
            context.line,
            context.offset + insert_at,
            context.offset + insert_at,
        ),
        requirement_prefix: format!(", {inserted_attr}: {quote}"),
        requirement_suffix: quote.to_string(),
    }
}

fn has_explicit_github_requirement(content: &str) -> bool {
    ["tag:", "ref:", "branch:"]
        .iter()
        .any(|attr| content.contains(attr))
}
