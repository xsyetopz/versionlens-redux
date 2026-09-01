use crate::positions::line_range;
use versionlens_model::Dependency;

use super::line::{GemLineContext, GemNameSpan, gem_name_range};
use super::syntax::attr_string_span;
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
    default_dependency(
        context,
        name,
        DefaultGithubSource {
            repo,
            repo_end,
            github_path: "commits",
            inserted_attr: "ref",
        },
    )
}

fn default_git_ref_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
) -> DefaultGithubDependency {
    let (repo, _, _, repo_end) = attr_string_span(context.content, "git")?;
    let repo = normalize_github_repository(repo.as_ref())?;
    default_dependency(
        context,
        name,
        DefaultGithubSource {
            repo,
            repo_end,
            github_path: "commits",
            inserted_attr: "ref",
        },
    )
}

fn default_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
    source: DefaultGithubSource<'_>,
) -> DefaultGithubDependency {
    let quote = context
        .content
        .as_bytes()
        .get(source.repo_end)
        .copied()
        .unwrap_or(b'"') as char;
    let insert_at = context.content.len();
    Some(Dependency {
        name: source.repo.to_owned(),
        requirement: "".to_owned(),
        ecosystem: Ruby,
        group: context.group.to_owned(),
        hosted_url: Some(github_api_url(source.repo, source.github_path)?),
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
        canonical_reference: None,
    })
}

fn has_explicit_github_requirement(content: &str) -> bool {
    ["tag:", "ref:", "branch:"]
        .iter()
        .any(|attr| content.contains(attr))
}

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
        hosted_url: Some(github_api_url(&repo, "commits")?),
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
        canonical_reference: None,
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

use std::borrow::Cow::{Borrowed as CowBorrowed, Owned as CowOwned};

use versionlens_model::GithubRepository;

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
    (GithubRepository::parse(value).is_some()
        && !owner.is_empty()
        && !repo.is_empty()
        && !repo.contains('/'))
    .then_some(value)
}

fn borrowed_attr_string<'a>(content: &'a str, name: &str) -> Option<&'a str> {
    let (value, _, _, _) = attr_string_span(content, name)?;
    match value {
        CowBorrowed(value) => Some(value),
        CowOwned(_) => None,
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
        hosted_url: Some(github_api_url(&repo, "tags")?),
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
        canonical_reference: None,
    })
}

pub(super) fn github_api_url(repo: &str, path: &str) -> Option<String> {
    GithubRepository::parse(repo)
        .map(|repository| repository.api_url("https://api.github.com/repos", &format!("/{path}")))
}
