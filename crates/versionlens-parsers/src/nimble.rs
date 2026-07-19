use crate::positions::offset_range;
use crate::quoted::double_quoted_string_at;
use crate::requirement_range::operator_requirement_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Nim;

const GITHUB_API_REPO_PREFIX: &str = "https://api.github.com/repos/";

pub(crate) fn parse_nimble(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut current_group = "requires".to_owned();
    let mut offset = 0usize;
    for line in text.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if indent == 0 {
            current_group = "requires".to_owned();
            if let Some(feature) = feature_block_name(trimmed) {
                current_group = format!("{feature}.requires");
            }
        }
        if let Some(relative) = trimmed.find("requires")
            && trimmed[..relative].trim().is_empty()
        {
            let requires_start = offset + indent + relative;
            if let Some(dependency) = parse_requires_line(text, requires_start, &current_group) {
                dependencies.push(dependency);
            }
        }
        offset += line.len() + 1;
    }
    dependencies
}

fn feature_block_name(line: &str) -> Option<&str> {
    if line == "dev:" {
        return Some("dev");
    }
    let rest = line.strip_prefix("feature ")?;
    let quote = rest.find('"')?;
    let value = &rest[quote + 1..];
    let end = value.find('"')?;
    Some(&value[..end])
}

fn parse_requires_line(text: &str, requires_start: usize, group: &str) -> Option<Dependency> {
    let quote_start = requires_start + text[requires_start..].find('"')?;
    let requirement = double_quoted_string_at(text, quote_start)?;
    let parsed = parse_requirement(requirement.value)?;
    let range = operator_requirement_range(
        &parsed.requirement,
        &["==", ">=", "<=", "^=", "~=", ">", "<"],
    );
    let requirement_start = requirement.start + parsed.requirement_start + range.start;
    let requirement_end = requirement.start + parsed.requirement_start + range.end;
    let mut dependency = Dependency {
        name: parsed.name,
        requirement: parsed.requirement,
        ecosystem: Nim,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, requirement.start, requirement.start + parsed.name_len),
        requirement_range: offset_range(text, requirement_start, requirement_end),
        requirement_prefix: range.prefix,
        requirement_suffix: "".to_owned(),
    };
    if let Some(repo) = parsed.github_repo {
        dependency.hosted_name = Some(repo.to_owned());
        dependency.hosted_url = Some(format!("{GITHUB_API_REPO_PREFIX}{repo}/tags"));
    }
    if parsed.is_head {
        dependency.hosted_url = Some("head".to_owned());
    }
    Some(dependency)
}

struct ParsedRequirement<'a> {
    name: String,
    name_len: usize,
    requirement: String,
    requirement_start: usize,
    github_repo: Option<&'a str>,
    is_head: bool,
}

fn parse_requirement(value: &str) -> Option<ParsedRequirement<'_>> {
    if value.starts_with("http://") || value.starts_with("https://") {
        return parse_url_requirement(value);
    }

    let (head, branch) = value
        .split_once('#')
        .map_or((value, None), |(name, branch)| {
            (name, (!branch.is_empty()).then_some(branch))
        });
    let mut parts = head.split_whitespace();
    let name = parts.next()?;
    let (requirement_start, requirement) = if let Some(branch) = branch {
        let start = head.len() + 1;
        (start, branch.to_owned())
    } else if let Some(relative) = head[name.len()..].find(|ch: char| !ch.is_whitespace()) {
        let start = name.len() + relative;
        (start, value[start..].trim().to_owned())
    } else {
        (name.len(), "".to_owned())
    };
    Some(ParsedRequirement {
        name: name.to_owned(),
        name_len: name.len(),
        requirement,
        requirement_start,
        github_repo: None,
        is_head: branch == Some("head"),
    })
}

fn parse_url_requirement(value: &str) -> Option<ParsedRequirement<'_>> {
    let mut parts = value.split_whitespace();
    let url_token = parts.next()?;
    let (url, branch) = url_token
        .split_once('#')
        .map_or((url_token, None), |(url, branch)| (url, Some(branch)));
    let name = package_name_from_url(url);
    let (requirement_start, requirement) =
        if let Some(relative) = value[url_token.len()..].find(|ch: char| !ch.is_whitespace()) {
            let start = url_token.len() + relative;
            (start, value[start..].trim().to_owned())
        } else if let Some(branch) = branch {
            let start = url.len() + 1;
            (start, branch.to_owned())
        } else {
            (url.len(), "".to_owned())
        };
    Some(ParsedRequirement {
        name,
        name_len: url.len(),
        requirement,
        requirement_start,
        github_repo: github_repo(url),
        is_head: false,
    })
}

fn package_name_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(url)
        .trim_end_matches(".git")
        .to_owned()
}

fn github_repo(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))?;
    let repo = rest.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = repo.split('/');
    let owner = parts.next()?;
    let name = parts.next()?;
    (parts.next().is_none() && !owner.is_empty() && !name.is_empty()).then_some(repo)
}
