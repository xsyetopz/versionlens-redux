use crate::positions::line_range;
use versionlens_model::Ecosystem::GitHub;
use versionlens_model::{Dependency, GithubRepository};

pub(crate) fn parse_github_actions(text: &str) -> Vec<Dependency> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| parse_uses_line(line_index, line))
        .collect()
}

fn parse_uses_line(line_index: usize, line: &str) -> Option<Dependency> {
    let trimmed = line.trim_start();
    let raw_value = trimmed
        .strip_prefix("uses:")
        .or_else(|| trimmed.strip_prefix("- uses:"))?
        .trim();
    let raw_value = raw_value
        .split_once(" #")
        .map_or(raw_value, |(value, _)| value)
        .trim();
    let (value, value_start) = unquote_value(raw_value, line.find(raw_value)?)?;
    let (name, raw_requirement) = value.split_once('@')?;
    let name = name.trim();
    let raw_requirement = raw_requirement.trim();
    let Some(repository) = github_repository_name(name) else {
        return None;
    };
    if !is_version_ref(raw_requirement) {
        return None;
    }
    let requirement = raw_requirement.trim_start_matches(['v', 'V']);
    let requirement_prefix = if requirement.len() == raw_requirement.len() {
        ""
    } else {
        "v"
    };

    let requirement_start = value_start + name.len() + 1;
    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: GitHub,
        group: "uses".to_owned(),
        hosted_url: None,
        hosted_name: Some(repository.to_owned()),
        range: line_range(line_index, line, value_start, value_start + name.len()),
        requirement_range: line_range(
            line_index,
            line,
            requirement_start,
            requirement_start + raw_requirement.len(),
        ),
        requirement_prefix: requirement_prefix.to_owned(),
        requirement_suffix: "".to_owned(),
    })
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

fn is_version_ref(value: &str) -> bool {
    let value = value.trim_start_matches(['v', 'V']);
    !is_commit_sha(value)
        && !value.is_empty()
        && value.as_bytes().first().is_some_and(u8::is_ascii_digit)
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'.' | b'-'
                        | b'+'
                        | b'^'
                        | b'~'
                        | b'<'
                        | b'>'
                        | b'='
                        | b'*'
                        | b'|'
                        | b','
                        | b' '
                )
        })
}

fn is_commit_sha(value: &str) -> bool {
    value.len() >= 7 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
#[cfg(test)]
mod inline_tests;
