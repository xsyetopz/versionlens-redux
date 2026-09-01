use crate::positions::line_range;
use versionlens_model::Ecosystem::GitHub;
use versionlens_model::{CanonicalReference, Dependency, GithubRepository};
use versionlens_versions::version_tag_parts;

pub(crate) fn parse_github_actions(text: &str) -> Vec<Dependency> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| parse_uses_line(line_index, line))
        .collect()
}

fn parse_uses_line(line_index: usize, line: &str) -> Option<Dependency> {
    let trimmed = line.trim_start();
    let value_with_comment = trimmed
        .strip_prefix("uses:")
        .or_else(|| trimmed.strip_prefix("- uses:"))?
        .trim_start();
    let value_with_comment_start = line.len() - value_with_comment.len();
    let comment_start = yaml_comment_start(value_with_comment);
    let raw_value = comment_start
        .map_or(value_with_comment, |start| &value_with_comment[..start])
        .trim_end();
    let (value, value_start) = unquote_value(raw_value, value_with_comment_start)?;
    let (name, raw_requirement) = value.split_once('@')?;
    let name = name.trim();
    let raw_requirement = raw_requirement.trim();
    let Some(repository) = github_repository_name(name) else {
        return None;
    };
    let requirement_start = value_start + name.len() + 1;
    let (requirement, requirement_prefix, requirement_end, canonical_reference) =
        if is_commit_sha(raw_requirement) {
            let comment_start = comment_start?;
            let comment = &value_with_comment[comment_start + 1..];
            let trimmed_comment = comment.trim_start();
            let annotation_space = comment.len() - trimmed_comment.len();
            let annotation = trimmed_comment.split_whitespace().next()?;
            let annotation_start = value_with_comment_start + comment_start + 1 + annotation_space;
            let (prefix, requirement) = version_tag_parts(annotation)?;
            let revision_end = requirement_start + raw_requirement.len();
            let annotation_end = annotation_start + annotation.len();
            (
                requirement,
                prefix,
                annotation_end,
                CanonicalReference::GitHubActionSha {
                    commit: raw_requirement.to_owned(),
                    tag: annotation.to_owned(),
                    separator: line.get(revision_end..annotation_start)?.to_owned(),
                },
            )
        } else {
            let (prefix, requirement) = version_tag_parts(raw_requirement)?;
            (
                requirement,
                prefix,
                requirement_start + raw_requirement.len(),
                CanonicalReference::GitHubActionTag {
                    tag: raw_requirement.to_owned(),
                },
            )
        };

    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: GitHub,
        group: "uses".to_owned(),
        hosted_url: None,
        hosted_name: Some(repository.to_owned()),
        range: line_range(line_index, line, value_start, value_start + name.len()),
        requirement_range: line_range(line_index, line, requirement_start, requirement_end),
        requirement_prefix: requirement_prefix.to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: Some(canonical_reference),
    })
}

fn yaml_comment_start(value: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quote == Some('"') && character == '\\' {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if character == '#'
            && quote.is_none()
            && value[..index]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace)
        {
            return Some(index);
        }
    }
    None
}

fn unquote_value(value: &str, value_start: usize) -> Option<(&str, usize)> {
    let quote = value.as_bytes().first().copied()?;
    if !matches!(quote, b'\'' | b'\"') {
        return Some((value, value_start));
    }
    (value.len() >= 2 && value.as_bytes().last().copied() == Some(quote))
        .then(|| (&value[1..value.len() - 1], value_start + 1))
}

fn github_repository_name(value: &str) -> Option<&str> {
    let mut parts = value.split('/');
    let Some(owner) = parts.next() else {
        return None;
    };
    let Some(repository) = parts.next() else {
        return None;
    };
    let identity = format!("{owner}/{repository}");
    if parts.any(|part| part.is_empty() || part == "." || part == "..") {
        return None;
    }
    GithubRepository::parse(&identity)
        .is_some()
        .then_some(&value[..owner.len() + 1 + repository.len()])
}

fn is_commit_sha(value: &str) -> bool {
    value.len() >= 7 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
#[cfg(test)]
mod inline_tests;
