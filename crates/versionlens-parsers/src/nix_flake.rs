use crate::positions::offset_range;
use crate::scanner::matching_delimiter;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Nix;

const GITHUB_API_REPO_PREFIX: &str = "https://api.github.com/repos/";

pub(crate) fn parse_nix_flake_with_paths(text: &str, dependency_paths: &[&str]) -> Vec<Dependency> {
    if !dependency_paths.is_empty() && !dependency_paths.contains(&"inputs") {
        return vec![];
    }

    let Some(inputs_start) = text.find("inputs") else {
        return vec![];
    };
    let Some(open) = text[inputs_start..]
        .find('{')
        .map(|index| inputs_start + index)
    else {
        return vec![];
    };
    let Some(close) = matching_delimiter(text, open, b'{', b'}') else {
        return vec![];
    };

    let mut dependencies = vec![];
    collect_input_urls(text, open + 1, close, &mut dependencies);
    dependencies
}

fn collect_input_urls(text: &str, start: usize, end: usize, out: &mut Vec<Dependency>) {
    let mut offset = start;
    while let Some(relative) = text[offset..end].find("url") {
        let url_start = offset + relative;
        let Some(input_alias) = input_alias_for_url(text, start, url_start) else {
            offset = url_start + 3;
            continue;
        };
        let equals = skip_ws(text, url_start + 3);
        if text.as_bytes().get(equals) != Some(&b'=') {
            offset = url_start + 3;
            continue;
        }
        let value_quote = skip_ws(text, equals + 1);
        if text.as_bytes().get(value_quote) != Some(&b'"') {
            offset = url_start + 3;
            continue;
        }
        let Some(value) = quoted_value(text, value_quote) else {
            break;
        };
        if let Some(dependency) = flake_input_dependency(text, &input_alias, &value) {
            out.push(dependency);
        }
        offset = value.outer_end;
    }
}

fn flake_input_dependency(text: &str, alias: &str, value: &QuotedValue<'_>) -> Option<Dependency> {
    let source = value.value.trim();
    let parsed = parse_flake_source(alias, source, value.value_start)?;
    Some(Dependency {
        name: parsed.name,
        requirement: parsed.requirement,
        ecosystem: Nix,
        group: "inputs".to_owned(),
        hosted_url: parsed.hosted_url,
        hosted_name: Some(alias.to_owned()),
        range: offset_range(text, value.value_start, value.value_end),
        requirement_range: offset_range(text, parsed.requirement_start, parsed.requirement_end),
        requirement_prefix: parsed.requirement_prefix,
        requirement_suffix: parsed.requirement_suffix,
    })
}

struct ParsedFlakeSource {
    name: String,
    requirement: String,
    hosted_url: Option<String>,
    requirement_start: usize,
    requirement_end: usize,
    requirement_prefix: String,
    requirement_suffix: String,
}

fn parse_flake_source(alias: &str, source: &str, source_start: usize) -> Option<ParsedFlakeSource> {
    if let Some(github) = source.strip_prefix("github:") {
        return parse_github_source(github, source_start + "github:".len());
    }
    if source.starts_with("path:") || source.starts_with("git+file:") || source.starts_with("file:")
    {
        return Some(ParsedFlakeSource {
            name: alias.to_owned(),
            requirement: source.to_owned(),
            hosted_url: Some("path".to_owned()),
            requirement_start: source_start,
            requirement_end: source_start + source.len(),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        });
    }
    if source.starts_with("git+") || source.starts_with("git:") || source.contains("://") {
        return Some(ParsedFlakeSource {
            name: alias.to_owned(),
            requirement: source.to_owned(),
            hosted_url: Some("git".to_owned()),
            requirement_start: source_start,
            requirement_end: source_start + source.len(),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        });
    }
    None
}

fn parse_github_source(source: &str, source_start: usize) -> Option<ParsedFlakeSource> {
    let query_start = source.find('?');
    let path = query_start.map_or(source, |index| &source[..index]);
    let query = query_start.map(|index| &source[index + 1..]);
    let mut parts = path.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    let path_ref = parts.collect::<Vec<_>>().join("/");
    let query_ref = query.and_then(query_ref);
    let raw_requirement = if let Some(query_ref) = query_ref {
        query_ref.to_owned()
    } else if !path_ref.is_empty() {
        path_ref
    } else {
        "latest".to_owned()
    };
    let (requirement, requirement_offset_delta) = nix_requirement_segment(&raw_requirement);
    let raw_requirement_offset = source.find(&raw_requirement).unwrap_or(0);
    let requirement_start = source_start + raw_requirement_offset + requirement_offset_delta;
    let requirement_end = requirement_start + requirement.len();
    let name = format!("{owner}/{repo}");
    Some(ParsedFlakeSource {
        hosted_url: Some(format!("{GITHUB_API_REPO_PREFIX}{name}/tags")),
        name,
        requirement: requirement.to_owned(),
        requirement_start,
        requirement_end,
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn nix_requirement_segment(requirement: &str) -> (&str, usize) {
    for prefix in ["nixos-", "release-"] {
        if let Some(rest) = requirement.strip_prefix(prefix)
            && rest
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_digit())
        {
            return (rest, prefix.len());
        }
    }
    (requirement, 0)
}

fn query_ref(query: &str) -> Option<&str> {
    query.split('&').find_map(|part| {
        part.strip_prefix("ref=")
            .or_else(|| part.strip_prefix("rev="))
    })
}

fn input_alias_for_url(text: &str, inputs_start: usize, url_start: usize) -> Option<String> {
    let prefix = text.get(inputs_start..url_start)?;
    let line_start = prefix
        .rfind('\n')
        .map(|index| inputs_start + index + 1)
        .unwrap_or(inputs_start);
    let line_prefix = text.get(line_start..url_start)?.trim_end();
    if let Some(alias) = line_prefix.strip_suffix('.') {
        let alias = alias.rsplit('.').next()?.trim();
        if !alias.is_empty() && alias != "inputs" {
            return Some(alias.to_owned());
        }
    }

    let before_line = text.get(inputs_start..line_start.saturating_sub(1))?;
    let mut depth = 0isize;
    for line in before_line.lines().rev() {
        let trimmed = line.trim();
        depth += trimmed.matches('}').count().cast_signed();
        depth -= trimmed.matches('{').count().cast_signed();
        if depth < 0 {
            if let Some((alias, _)) = trimmed.split_once('=') {
                let alias = alias.trim();
                if !alias.is_empty() && !alias.contains('.') {
                    return Some(alias.to_owned());
                }
            }
            break;
        }
    }
    None
}

struct QuotedValue<'a> {
    value: &'a str,
    value_start: usize,
    value_end: usize,
    outer_end: usize,
}

fn quoted_value(text: &str, quote: usize) -> Option<QuotedValue<'_>> {
    let mut offset = quote + 1;
    let mut escaped = false;
    while offset < text.len() {
        let byte = *text.as_bytes().get(offset)?;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Some(QuotedValue {
                value: text.get(quote + 1..offset)?,
                value_start: quote + 1,
                value_end: offset,
                outer_end: offset + 1,
            });
        }
        offset += 1;
    }
    None
}

fn skip_ws(text: &str, mut offset: usize) -> usize {
    while text
        .as_bytes()
        .get(offset)
        .is_some_and(|value| value.is_ascii_whitespace())
    {
        offset += 1;
    }
    offset
}
