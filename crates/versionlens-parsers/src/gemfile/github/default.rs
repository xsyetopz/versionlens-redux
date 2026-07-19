use crate::positions::line_range;
use versionlens_model::Dependency;

use super::super::line::{GemLineContext, GemNameSpan, gem_name_range};
use super::super::syntax::attr_string_span;
use super::repository::normalize_github_repository;
use super::url::github_api_url;
use versionlens_model::Ecosystem::Ruby;

type DefaultGithubDependency = Option<Dependency>;

struct DefaultGithubSource<'a> {
    repo: &'a str,
    repo_end: usize,
    github_path: &'a str,
    inserted_attr: &'a str,
}

pub(in crate::gemfile) fn gem_github_default_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
) -> DefaultGithubDependency {
    if has_explicit_github_requirement(context.content) {
        return None;
    }

    default_github_ref_dependency(context, name)
        .or_else(|| default_git_ref_dependency(context, name))
}

fn default_github_ref_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
) -> DefaultGithubDependency {
    let (repo, _, _, repo_end) = attr_string_span(context.content, "github")?;
    let repo = normalize_github_repository(repo.as_ref())?;
    Some(default_dependency(
        context,
        name,
        DefaultGithubSource {
            repo,
            repo_end,
            github_path: "commits",
            inserted_attr: "ref",
        },
    ))
}

fn default_git_ref_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
) -> DefaultGithubDependency {
    let (repo, _, _, repo_end) = attr_string_span(context.content, "git")?;
    let repo = normalize_github_repository(repo.as_ref())?;
    Some(default_dependency(
        context,
        name,
        DefaultGithubSource {
            repo,
            repo_end,
            github_path: "commits",
            inserted_attr: "ref",
        },
    ))
}

fn default_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
    source: DefaultGithubSource<'_>,
) -> Dependency {
    let quote = context
        .content
        .as_bytes()
        .get(source.repo_end)
        .copied()
        .unwrap_or(b'"') as char;
    let insert_at = context.content.len();
    Dependency {
        name: source.repo.to_owned(),
        requirement: "".to_owned(),
        ecosystem: Ruby,
        group: context.group.to_owned(),
        hosted_url: Some(github_api_url(source.repo, source.github_path)),
        hosted_name: Some(name.name.to_owned()),
        range: gem_name_range(context),
        requirement_range: line_range(
            context.line_index,
            context.line,
            context.offset + insert_at,
            context.offset + insert_at,
        ),
        requirement_prefix: format!(", {}: {quote}", source.inserted_attr),
        requirement_suffix: quote.to_string(),
    }
}

fn has_explicit_github_requirement(content: &str) -> bool {
    ["tag:", "ref:", "branch:"]
        .iter()
        .any(|attr| content.contains(attr))
}
