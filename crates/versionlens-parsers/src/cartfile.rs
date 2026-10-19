//! Carthage `Cartfile` and `Cartfile.private`. Each line names an origin
//! (`github "owner/repo"` or `git "url"`) and an optional version: an
//! operator requirement (`~> 1.0`, `>= 1.0`, `== 1.0`) or a quoted
//! branch, tag or commit.

use crate::github;
use crate::positions::line_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Swift;

const GITHUB_URL_PREFIXES: &[&str] = &[
    "https://github.com/",
    "http://github.com/",
    "git@github.com:",
];

pub(crate) fn parse_cartfile(text: &str) -> Vec<Dependency> {
    text.split('\n')
        .enumerate()
        .filter_map(|(index, line)| parse_line(index, line.trim_end_matches('\r')))
        .collect()
}

fn parse_line(index: usize, line: &str) -> Option<Dependency> {
    let code = line.split_once('#').map_or(line, |(code, _)| code);
    let kind_start = code.len() - code.trim_start().len();
    let (kind, _) = code[kind_start..].split_once(char::is_whitespace)?;
    // `binary` origins list their versions in a JSON spec, not in tags.
    if kind != "github" && kind != "git" {
        return None;
    }
    let rest_start = kind_start + kind.len() + 1;
    let (origin, origin_start, origin_end) = quoted(code, rest_start)?;
    let version_start = origin_end + 1;
    let version = code[version_start..].trim();
    if version.is_empty() {
        return None;
    }
    let requirement_start = version_start + code[version_start..].find(version)?;

    let repository = if kind == "github" {
        github::repository_from_url(origin, GITHUB_URL_PREFIXES)
            .or_else(|| github::repository_from_path(origin))
    } else {
        github::repository_from_url(origin, GITHUB_URL_PREFIXES)
    };
    let (requirement, start, end) = match quoted(code, requirement_start) {
        Some(reference) => reference,
        None => (
            version,
            requirement_start,
            requirement_start + version.len(),
        ),
    };
    let is_reference = start != requirement_start;
    let (hosted_name, hosted_url) = match repository {
        Some(repository) if !is_reference => (
            Some(repository.to_owned()),
            Some(github::tags_url(repository)?),
        ),
        _ => (None, Some("git".to_owned())),
    };

    Some(Dependency {
        name: origin.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Swift,
        group: "dependencies".to_owned(),
        hosted_url,
        hosted_name,
        range: line_range(index, line, origin_start, origin_end),
        requirement_range: line_range(index, line, start, end),
        requirement_prefix: if is_reference {
            String::new()
        } else {
            operator_prefix(requirement).to_owned()
        },
        requirement_suffix: String::new(),
        canonical_reference: None,
    })
}

/// The double-quoted string that starts at the first non-space byte at or
/// after `from`, with the byte range of its contents.
fn quoted(line: &str, from: usize) -> Option<(&str, usize, usize)> {
    let open = from + line[from..].find(|char: char| !char.is_whitespace())?;
    if !line[open..].starts_with('"') {
        return None;
    }
    let start = open + 1;
    let end = start + line[start..].find('"')?;
    Some((&line[start..end], start, end))
}

/// The operator and the space after it, such as `~> ` in `~> 1.0`.
fn operator_prefix(requirement: &str) -> &str {
    let operator_end = requirement
        .find(|char: char| !matches!(char, '~' | '>' | '=' | '<'))
        .unwrap_or(requirement.len());
    let rest = &requirement[operator_end..];
    &requirement[..operator_end + rest.len() - rest.trim_start().len()]
}
